//! Checked scalar subset of SharpRetro Tier0. One guest instruction per block.
use crate::{Result, memory, native::Code};
use sharpretro_jit::tier0::{StateLayout, Tier0};
use sharpretro_jit::{Builder, IlType, IntrinsicId, LocalId, NativeSlot, RegFile, RoundMode};
use std::cell::{Cell, RefCell};
use xfusion_recomp::{
    decode::XMode,
    disassembler::{DEF_MNEMONICS, decode_insn},
    lift::{FLAGS_ALL_LIVE, lift_one},
};

static LAYOUT: StateLayout = StateLayout {
    state_words: 91,
    off_pc: 17 * 8,
    off_membase: 88 * 8,
    flag_file: 1,
    off_flags: 16 * 8,
    flag_bit: |idx| idx,
    reg_off: |file, idx| match file.0 {
        0 if idx < 8 => idx * 8,
        2 if idx < 6 => (18 + idx) * 8,
        3 if idx < 8 => (24 + idx * 2) * 8, // eight legacy XMM registers
        _ => panic!("unadmitted register file/index: {file:?}/{idx}"),
    },
    off_rr: 0,
    gpr_w_zext: true,
};
const VARIABLE_SHIFTS: [&str; 8] = [
    "PSRLW", "PSRLD", "PSRLQ", "PSRAW", "PSRAD", "PSLLW", "PSLLD", "PSLLQ",
];
struct Stored {
    address: u32,
    value: u32,
    scopes: Vec<u64>,
}
struct Shared {
    guards: RefCell<Vec<usize>>,
    stored: Cell<bool>,
    last_store: RefCell<Option<Stored>>,
    scopes: RefCell<Vec<u64>>,
    next_scope: Cell<u64>,
    stack_store: Option<IlType>,
    stack_read: bool,
    stack_write: bool,
    segment: Option<u32>,
    signed_division: bool,
    aligned_vector: bool,
}
struct Checked<'a> {
    inner: &'a mut Tier0,
    shared: &'a Shared,
}
fn width(ty: IlType) -> u64 {
    match ty {
        IlType::I {
            width: 8 | 16 | 32 | 64,
            ..
        } => {
            if let IlType::I { width, .. } = ty {
                width as u64 / 8
            } else {
                unreachable!()
            }
        }
        IlType::V128 | IlType::I { width: 128, .. } => 16,
        IlType::F { width: 32 } => 4,
        IlType::F { width: 64 } => 8,
        _ => panic!("unadmitted memory type {ty:?}"),
    }
}
impl Checked<'_> {
    fn compare_string(&mut self, mnemonic: &str, width: u32) {
        let ty = xfusion_recomp::operand::ilty(width);
        let scan = mnemonic.starts_with("SCAS");
        let si = self.reg_read(RegFile(0), 6, IlType::U32);
        let di = self.reg_read(RegFile(0), 7, IlType::U32);
        let lhs = if scan {
            let a = self.reg_read(RegFile(0), 0, IlType::U32);
            self.cast(a, ty)
        } else {
            self.mem_read(si, ty)
        };
        let rhs = self.mem_read(di, ty);
        // Both reads precede architectural changes. A later REP fault retains
        // completed iterations and rolls back only this iteration.
        let result = self.sub(lhs, rhs);
        let carry = self.lt(lhs, rhs);
        self.reg_write(RegFile(1), 0, carry);
        let zero = self.literal(ty, 0);
        let zf = self.eq(result, zero);
        self.reg_write(RegFile(1), 6, zf);
        self.flag_bit(7, result, width - 1);
        let different = self.xor(lhs, rhs);
        let changed = self.xor(lhs, result);
        let overflow = self.and(different, changed);
        self.flag_bit(11, overflow, width - 1);
        let auxiliary = self.xor(different, result);
        self.flag_bit(4, auxiliary, 4);
        let low = self.cast(result, IlType::U8);
        let low = self.cast(low, IlType::U32);
        let four = self.literal(IlType::U32, 4);
        let high = self.shr(low, four);
        let folded = self.xor(low, high);
        let mask = self.literal(IlType::U32, 15);
        let folded = self.and(folded, mask);
        let table = self.literal(IlType::U32, 0x9669);
        let parity = self.shr(table, folded);
        self.flag_bit(2, parity, 0);
        let step = self.literal(IlType::U32, (width / 8) as u128);
        let negative = self.neg(step);
        let df = self.reg_read(RegFile(1), 10, IlType::Bool);
        let step = self.ternary(df, negative, step);
        if !scan {
            let next = self.add(si, step);
            self.reg_write(RegFile(0), 6, next);
        }
        let next = self.add(di, step);
        self.reg_write(RegFile(0), 7, next);
    }
    fn flag_bit(&mut self, flag: u32, value: u32, bit: u32) {
        let shift = self.literal(IlType::U32, bit as u128);
        let value = self.shr(value, shift);
        let one = self.literal(self.ty_of(value), 1);
        let value = self.and(value, one);
        let value = self.cast(value, IlType::Bool);
        self.reg_write(RegFile(1), flag, value);
    }
    fn address(&mut self, addr: u32, stack: bool) -> u32 {
        // Address-size wrap happens before adding the segment base. Upstream's
        // ModRM helper currently defers Bits32 masking; correct it at access.
        if !stack && let Some(si) = self.shared.segment {
            let segment = self.inner.reg_read(RegFile(2), si, IlType::U64);
            let offset = self.inner.sub(addr, segment);
            let offset = self.inner.cast(offset, IlType::U32);
            let offset = self.inner.cast(offset, IlType::U64);
            self.inner.add(segment, offset)
        } else {
            self.inner.cast(addr, IlType::U32)
        }
    }
    fn guard(&mut self, addr: u32, size: u64, write: bool) {
        assert!(addr < 4096, "spill limit");
        let e = &mut self.inner.enc;
        e.ldr_x(0, 28, 90 * 8); // checked memory context, never guest-controlled
        e.ldr_x(1, 27, addr * 8);
        e.mov_imm64(2, size);
        e.mov_imm64(3, write as u64);
        e.mov_imm64(16, memory::guard as *const () as u64);
        e.put_raw(0xd63f0200); // blr x16; Rust helper preserves x27/x28
        let at = e.here();
        e.cbz(0, 0); // failure goes straight to this instruction's epilogue
        self.shared.guards.borrow_mut().push(at);
    }
    fn vector_alignment(&mut self, addr: u32) {
        if !self.shared.aligned_vector {
            return;
        }
        let e = &mut self.inner.enc;
        e.ldr_x(0, 28, 90 * 8);
        e.ldr_x(1, 27, addr * 8);
        e.mov_imm64(16, memory::vector_alignment as *const () as u64);
        e.put_raw(0xd63f0200);
        let at = e.here();
        e.cbz(0, 0);
        self.shared.guards.borrow_mut().push(at);
    }
    fn atomic_address(&mut self, addr: u32, size: u64) -> u32 {
        assert!(
            !self.shared.stored.replace(true),
            "multiple committed writes"
        );
        let addr = self.address(addr, false);
        self.guard(addr, size, false);
        self.guard(addr, size, true);
        let e = &mut self.inner.enc;
        e.ldr_x(0, 28, 90 * 8);
        e.ldr_x(1, 27, addr * 8);
        e.mov_imm64(2, size);
        e.mov_imm64(16, memory::atomic_alignment as *const () as u64);
        e.put_raw(0xd63f0200);
        let at = e.here();
        e.cbz(0, 0);
        self.shared.guards.borrow_mut().push(at);
        addr
    }
    fn fp_control(&mut self, op: u64, value: u32) -> u32 {
        let result = self.inner.literal(IlType::U64, 0);
        let e = &mut self.inner.enc;
        e.ldr_x(0, 28, 90 * 8);
        e.mov_imm64(1, op);
        e.ldr_x(2, 27, value * 8);
        e.mov_imm64(16, crate::fpu::control as *const () as u64);
        e.put_raw(0xd63f0200);
        e.str_x(0, 27, result * 8);
        result
    }
    fn checked_val(&self, v: u32) -> u32 {
        let wide = matches!(
            self.inner.ty_of(v),
            IlType::V128 | IlType::I { width: 128, .. }
        );
        assert!(
            v.checked_add(if wide { 2 } else { 1 })
                .is_some_and(|n| n <= 4096),
            "spill limit"
        );
        v
    }
}
impl Builder for Checked<'_> {
    type Val = u32;
    fn ty_of(&self, v: u32) -> IlType {
        self.inner.ty_of(v)
    }
    fn mem_read(&mut self, addr: u32, ty: IlType) -> u32 {
        if self.shared.stored.get() {
            let saved = self.shared.last_store.borrow();
            let store = saved
                .as_ref()
                .filter(|store| {
                    store.address == addr
                        && width(self.inner.ty_of(store.value)) == width(ty)
                        && self.shared.scopes.borrow().starts_with(&store.scopes)
                })
                .expect("post-store read needs the same typed operand and a dominating store");
            let mut value = store.value;
            drop(saved);
            // A physical byte/word/dword store discards any upper spill bits.
            // Forward the reloaded bit pattern, not the full source register.
            if width(ty) < 8 {
                value = self.inner.bitcast(value, IlType::U64);
                let mask = self
                    .inner
                    .literal(IlType::U64, (1u128 << (width(ty) * 8)) - 1);
                value = self.inner.and(value, mask);
            }
            let value = self.inner.bitcast(value, ty);
            return self.checked_val(value);
        }
        let addr = self.address(addr, self.shared.stack_read);
        self.vector_alignment(addr);
        self.guard(addr, width(ty), false);
        let v = self.inner.mem_read(addr, ty);
        self.checked_val(v)
    }
    fn mem_write(&mut self, addr: u32, value: u32) {
        assert!(
            !self.shared.stored.replace(true),
            "multiple stores require precise partial-state support"
        );
        // Generated CALL currently uses a U64 return-PC literal even in Bits32.
        // Store exactly four guest bytes, as required by the 32-bit CALL ABI.
        let value = if let Some(ty) = self.shared.stack_store {
            self.inner.cast(value, ty)
        } else {
            value
        };
        let original = addr;
        let addr = self.address(addr, self.shared.stack_write);
        self.vector_alignment(addr);
        self.guard(addr, width(self.inner.ty_of(value)), true);
        self.inner.mem_write(addr, value);
        *self.shared.last_store.borrow_mut() = Some(Stored {
            address: original,
            value,
            scopes: self.shared.scopes.borrow().clone(),
        });
    }
    fn branch(&mut self, target: u32, link: bool) {
        assert!(!link, "guest native-link branch is not an x86 CALL");
        let target = self.inner.cast(target, IlType::U32);
        self.inner.branch(target, false);
    }
    fn branched(&self) -> bool {
        self.inner.branched()
    }
    fn cond(&mut self, c: u32, yes: &mut dyn FnMut(&mut Self), no: &mut dyn FnMut(&mut Self)) {
        self.inner.enc.ldr_x(9, 27, c * 8);
        let at = self.inner.enc.here();
        self.inner.enc.cbz(9, 0);
        let scope = self.shared.next_scope.get();
        self.shared
            .next_scope
            .set(scope.checked_add(2).expect("scope budget"));
        self.shared.scopes.borrow_mut().push(scope);
        yes(self);
        self.shared.scopes.borrow_mut().pop();
        let jump = self.inner.enc.here();
        self.inner.enc.b(0);
        let otherwise = self.inner.enc.here();
        self.shared.scopes.borrow_mut().push(scope + 1);
        no(self);
        self.shared.scopes.borrow_mut().pop();
        let end = self.inner.enc.here();
        assert!(
            otherwise - at < (1 << 18) && end - jump < (1 << 25),
            "conditional range"
        );
        self.inner
            .enc
            .patch(at, 0xb4000009 | (((otherwise - at) as u32) << 5));
        self.inner
            .enc
            .patch(jump, 0x14000000 | ((end - jump) as u32));
    }
    fn unimplemented(&mut self, name: &'static str) {
        panic!("unsupported guest instruction: {name}");
    }
    fn set_insn_pc(&mut self, _pc: u64) {}
    fn set_trace_next(&mut self, _pc: Option<u64>) {}
    fn trace_take_elided(&mut self) -> bool {
        false
    }
    fn literal(&mut self, ty: IlType, bits: u128) -> Self::Val {
        // The x86 decoder sign-extends encoded immediates to i64, and the
        // operand binder passes those bits as u128. Tier0 materializes literal
        // containers without masking. Canonicalize to the IL width here, as its
        // register/memory producers do, before unsigned carry comparisons see
        // the value. In particular U32(0xfffe0200) is not 0xfffffffffffe0200.
        let bits = match ty {
            IlType::I {
                width: width @ 1..=127,
                ..
            } => bits & ((1u128 << width) - 1),
            _ => bits,
        };
        let v = self.inner.literal(ty, bits);
        self.checked_val(v)
    }
    fn reg_read(&mut self, file: RegFile, idx: u32, ty: IlType) -> Self::Val {
        let v = self.inner.reg_read(file, idx, ty);
        self.checked_val(v)
    }
    fn reg_write(&mut self, file: RegFile, idx: u32, v: Self::Val) {
        let v = if file.0 == 0 {
            self.inner.cast(v, IlType::U32)
        } else {
            v
        };
        self.inner.reg_write(file, idx, v)
    }
    fn add(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.add(a, b);
        self.checked_val(v)
    }
    fn sub(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.sub(a, b);
        self.checked_val(v)
    }
    fn mul(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.mul(a, b);
        self.checked_val(v)
    }
    fn div(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        assert_eq!(
            width(self.inner.ty_of(a)),
            8,
            "only 32-bit DIV/IDIV operands admitted"
        );
        let e = &mut self.inner.enc;
        e.ldr_x(0, 28, 90 * 8);
        e.ldr_x(1, 27, a * 8);
        e.ldr_x(2, 27, b * 8);
        e.mov_imm64(3, self.shared.signed_division as u64);
        e.mov_imm64(16, memory::divide_guard as *const () as u64);
        e.put_raw(0xd63f0200);
        let at = e.here();
        e.cbz(0, 0);
        self.shared.guards.borrow_mut().push(at);
        let v = self.inner.div(a, b);
        self.checked_val(v)
    }
    fn rem(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.rem(a, b);
        self.checked_val(v)
    }
    fn neg(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.neg(a);
        self.checked_val(v)
    }
    fn and(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.and(a, b);
        self.checked_val(v)
    }
    fn or(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.or(a, b);
        self.checked_val(v)
    }
    fn xor(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.xor(a, b);
        self.checked_val(v)
    }
    fn not(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.not(a);
        self.checked_val(v)
    }
    fn shl(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.shl(a, b);
        self.checked_val(v)
    }
    fn shr(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.shr(a, b);
        self.checked_val(v)
    }
    fn rotr(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.rotr(a, b);
        self.checked_val(v)
    }
    fn rbit(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.rbit(a);
        self.checked_val(v)
    }
    fn clz(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.clz(a);
        self.checked_val(v)
    }
    fn popcnt(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.popcnt(a);
        self.checked_val(v)
    }
    fn bswap(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.bswap(a);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn vdpp(&mut self, a: Self::Val, b: Self::Val, imm: u32, ew: u32) -> Self::Val {
        panic!("unsupported JIT operation: vdpp")
    }
    fn eq(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.eq(a, b);
        self.checked_val(v)
    }
    fn ne(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.ne(a, b);
        self.checked_val(v)
    }
    fn lt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.lt(a, b);
        self.checked_val(v)
    }
    fn le(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.le(a, b);
        self.checked_val(v)
    }
    fn gt(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.gt(a, b);
        self.checked_val(v)
    }
    fn ge(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.ge(a, b);
        self.checked_val(v)
    }
    fn cast(&mut self, a: Self::Val, to: IlType) -> Self::Val {
        let v = self.inner.cast(a, to);
        self.checked_val(v)
    }
    fn bitcast(&mut self, a: Self::Val, to: IlType) -> Self::Val {
        let v = self.inner.bitcast(a, to);
        self.checked_val(v)
    }
    fn sext(&mut self, a: Self::Val, to: IlType) -> Self::Val {
        let v = self.inner.sext(a, to);
        self.checked_val(v)
    }
    fn pair128(&mut self, hi: Self::Val, lo: Self::Val) -> Self::Val {
        let v = self.inner.pair128(hi, lo);
        self.checked_val(v)
    }
    fn lo64(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.lo64(a);
        self.checked_val(v)
    }
    fn hi64(&mut self, a: Self::Val) -> Self::Val {
        let v = self.inner.hi64(a);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn vzip(&mut self, a: Self::Val, b: Self::Val, elem_bits: u32, hi: bool) -> Self::Val {
        let v = self.inner.vzip(a, b, elem_bits, hi);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn vfbin(&mut self, a: Self::Val, b: Self::Val, elem_bits: u32, op: u32) -> Self::Val {
        panic!("unsupported JIT operation: vfbin")
    }
    #[allow(unused_variables)]
    fn vibin(&mut self, a: Self::Val, b: Self::Val, elem_bits: u32, op: u32) -> Self::Val {
        let v = self.inner.vibin(a, b, elem_bits, op);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn vishi(&mut self, a: Self::Val, ew: u32, count: u32, dir: u32) -> Self::Val {
        let v = self.inner.vishi(a, ew, count, dir);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn vmovmsk(&mut self, a: Self::Val, ew: u32) -> Self::Val {
        panic!("unsupported JIT operation: vmovmsk")
    }
    #[allow(unused_variables)]
    fn vfun(&mut self, a: Self::Val, ew: u32, op: u32) -> Self::Val {
        panic!("unsupported JIT operation: vfun")
    }
    #[allow(unused_variables)]
    fn vfminmax(&mut self, a: Self::Val, b: Self::Val, ew: u32, is_max: bool) -> Self::Val {
        panic!("unsupported JIT operation: vfminmax")
    }
    #[allow(unused_variables)]
    fn vfcmpp(&mut self, a: Self::Val, b: Self::Val, ew: u32, pred: u32) -> Self::Val {
        panic!("unsupported JIT operation: vfcmpp")
    }
    #[allow(unused_variables)]
    fn vhadd(&mut self, a: Self::Val, b: Self::Val, ew: u32) -> Self::Val {
        panic!("unsupported JIT operation: vhadd")
    }
    #[allow(unused_variables)]
    fn vshuf(&mut self, a: Self::Val, b: Self::Val, elem_bits: u32, sel: u32) -> Self::Val {
        let v = self.inner.vshuf(a, b, elem_bits, sel);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn vshufw(&mut self, src: Self::Val, sel: u32, hi: bool) -> Self::Val {
        panic!("unsupported JIT operation: vshufw")
    }
    #[allow(unused_variables)]
    fn vcvt(&mut self, a: Self::Val, kind: u32) -> Self::Val {
        assert_eq!(kind, 4, "only exact packed i32-to-f64 conversion admitted");
        let a = self.inner.cast(a, IlType::V128);
        self.checked_val(a);
        let v = self.inner.vcvt(a, kind);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn fabs(&mut self, a: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: fabs")
    }
    #[allow(unused_variables)]
    fn fsqrt(&mut self, a: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: fsqrt")
    }
    #[allow(unused_variables)]
    fn fround(&mut self, a: Self::Val, mode: RoundMode) -> Self::Val {
        panic!("unsupported JIT operation: fround")
    }
    #[allow(unused_variables)]
    fn fceil(&mut self, a: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: fceil")
    }
    #[allow(unused_variables)]
    fn ffloor(&mut self, a: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: ffloor")
    }
    #[allow(unused_variables)]
    fn fisnan(&mut self, a: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: fisnan")
    }
    #[allow(unused_variables)]
    fn fcmpp(&mut self, a: Self::Val, b: Self::Val, pred: u32, w: u32) -> Self::Val {
        panic!("unsupported JIT operation: fcmpp")
    }
    #[allow(unused_variables)]
    fn fminmax(&mut self, a: Self::Val, b: Self::Val, is_max: bool) -> Self::Val {
        panic!("unsupported JIT operation: fminmax")
    }
    #[allow(unused_variables)]
    fn velement_read(&mut self, v: Self::Val, i: Self::Val, elem_ty: IlType) -> Self::Val {
        let bits = width(elem_ty) * 8;
        assert!([8, 16, 32, 64].contains(&bits));
        let size = self.inner.literal(IlType::U32, bits as u128);
        let shift = self.inner.mul(i, size);
        let sixty_four = self.inner.literal(IlType::U32, 64);
        let high = self.inner.and(shift, sixty_four);
        let zero = self.inner.literal(IlType::U32, 0);
        let high = self.inner.ne(high, zero);
        let lo = self.inner.lo64(v);
        let hi = self.inner.hi64(v);
        let word = self.inner.ternary(high, hi, lo);
        let sixty_three = self.inner.literal(IlType::U32, 63);
        let shift = self.inner.and(shift, sixty_three);
        let word = self.inner.shr(word, shift);
        let result = self.inner.cast(word, elem_ty);
        self.checked_val(result)
    }
    #[allow(unused_variables)]
    fn velement_write(&mut self, v: Self::Val, i: Self::Val, e: Self::Val) -> Self::Val {
        let bits = width(self.inner.ty_of(e)) * 8;
        assert!([8, 16, 32, 64].contains(&bits));
        let size = self.inner.literal(IlType::U32, bits as u128);
        let shift = self.inner.mul(i, size);
        let sixty_four = self.inner.literal(IlType::U32, 64);
        let high = self.inner.and(shift, sixty_four);
        let zero = self.inner.literal(IlType::U32, 0);
        let high = self.inner.ne(high, zero);
        let lo = self.inner.lo64(v);
        let hi = self.inner.hi64(v);
        let word = self.inner.ternary(high, hi, lo);
        let sixty_three = self.inner.literal(IlType::U32, 63);
        let shift = self.inner.and(shift, sixty_three);
        let mask = self.inner.literal(IlType::U64, (1u128 << bits) - 1);
        let mask = self.inner.shl(mask, shift);
        let inverse = self.inner.not(mask);
        let kept = self.inner.and(word, inverse);
        let elem = self.inner.cast(e, IlType::U64);
        let elem = self.inner.shl(elem, shift);
        let elem = self.inner.and(elem, mask);
        let merged = self.inner.or(kept, elem);
        let low = self.inner.ternary(high, lo, merged);
        let high = self.inner.ternary(high, merged, hi);
        let result = self.inner.pair128(high, low);
        self.checked_val(result)
    }
    #[allow(unused_variables)]
    fn vzero_top(&mut self, v: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: vzero_top")
    }
    #[allow(unused_variables)]
    fn mem_rmw_atomic(
        &mut self,
        _op: u8,
        _addr: Self::Val,
        _val: Self::Val,
        _ty: IlType,
    ) -> Self::Val {
        let addr = self.atomic_address(_addr, width(_ty));
        let v = self.inner.mem_rmw_atomic(_op, addr, _val, _ty);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn fence(&mut self) {
        self.inner.fence();
    }
    #[allow(unused_variables)]
    fn mem_cas_atomic(
        &mut self,
        _addr: Self::Val,
        _expected: Self::Val,
        _new: Self::Val,
        _ty: IlType,
    ) -> Self::Val {
        let addr = self.atomic_address(_addr, width(_ty));
        let v = self.inner.mem_cas_atomic(addr, _expected, _new, _ty);
        self.checked_val(v)
    }
    #[allow(unused_variables)]
    fn loop_n(&mut self, n: Self::Val, body: &mut dyn FnMut(&mut Self)) {
        panic!("unsupported JIT operation: loop_n")
    }
    #[allow(unused_variables)]
    fn loop_while(
        &mut self,
        count: Self::Val,
        exit_on: bool,
        body: &mut dyn FnMut(&mut Self) -> Self::Val,
    ) -> Self::Val {
        panic!("unsupported JIT operation: loop_while")
    }
    fn ternary(&mut self, c: Self::Val, a: Self::Val, b: Self::Val) -> Self::Val {
        let v = self.inner.ternary(c, a, b);
        self.checked_val(v)
    }
    fn local_new(&mut self, ty: IlType) -> LocalId {
        self.inner.local_new(ty)
    }
    fn local_read(&mut self, l: LocalId) -> Self::Val {
        let v = self.inner.local_read(l);
        self.checked_val(v)
    }
    fn local_write(&mut self, l: LocalId, v: Self::Val) {
        self.inner.local_write(l, v)
    }
    #[allow(unused_variables)]
    fn call_native(&mut self, slot: NativeSlot, args: &[Self::Val]) -> Option<Self::Val> {
        panic!("unsupported JIT operation: call_native")
    }
    #[allow(unused_variables)]
    fn call_intrinsic(&mut self, id: IntrinsicId, args: &[Self::Val]) -> Option<Self::Val> {
        if id == IntrinsicId(343) && args.is_empty() {
            let e = &mut self.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_r(1, 28);
            e.mov_imm64(16, crate::fpu::cpuid as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            self.shared.guards.borrow_mut().push(at);
            None
        } else {
            panic!("unsupported JIT intrinsic {}", id.0)
        }
    }
}

pub(crate) struct Instruction {
    pub code: Code,
    pub length: usize,
}
pub(crate) fn compile(pc: u32, bytes: &[u8]) -> Result<Instruction> {
    if std::env::var_os("XF_WATCH").is_some() {
        return Err(
            "SharpRetro watch instrumentation is outside the checked runner contract".into(),
        );
    }
    // The pinned decoder has no RDTSC definition. Admit only its exact plain
    // two-byte encoding; retain explicit failures for other missing encodings.
    let bank = match bytes {
        [0x60, ..] => Some((0, 1)),
        [0x61, ..] => Some((1, 1)),
        [0x66, 0x60, ..] => Some((2, 2)),
        [0x66, 0x61, ..] => Some((3, 2)),
        _ => None,
    };
    let rdtsc = bytes.starts_with(&[0x0f, 0x31]);
    let d = if let Some((_, len)) = bank {
        xfusion_recomp::decode::DecodedInsn {
            len,
            ..Default::default()
        }
    } else if rdtsc {
        xfusion_recomp::decode::DecodedInsn {
            len: 2,
            op: 0x31,
            ..Default::default()
        }
    } else {
        decode_insn(bytes, XMode::Bits32).ok_or_else(|| format!("decode failure at {pc:#x}"))?
    };
    let mnemonic = if let Some((op, _)) = bank {
        if op & 1 == 0 { "PUSHA" } else { "POPA" }
    } else if rdtsc {
        "RDTSC"
    } else {
        DEF_MNEMONICS[d.def_id as usize]
    };
    // String loops, FP/SIMD, interrupts, system instructions and implicit memory
    // operations are outside the initial admitted subset. Never execute an empty
    // generated template as if it implemented a guest operation.
    const ADMITTED: &[&str] = &[
        "CMPXCHG",
        "XADD",
        "CDQ",
        "CWDE",
        "SAHF",
        "LAHF",
        "RDTSC",
        "PUSHF",
        "POPF",
        "CPUID",
        "FWAIT",
        "FNCLEX",
        "FNINIT",
        "FNSTCW",
        "FNSTSW-M",
        "FNSTSW-AX",
        "FLDCW",
        "XCHG",
        "CLD",
        "STD",
        "MOVSB",
        "MOVSV",
        "STOSB",
        "STOSV",
        "LODSB",
        "LODSV",
        "CMPSB",
        "CMPSV",
        "SCASB",
        "SCASV",
        "DIV",
        "IDIV",
        "MUL",
        "IMUL",
        "IMUL3",
        "IMUL1",
        "LDMXCSR",
        "STMXCSR",
        "CVTTSD2SI",
        "CVTTSS2SI",
        "CVTSD2SI",
        "CVTSS2SI",
        "CVTSI2SD",
        "CVTSI2SS",
        "MOVLPD",
        "PSRLQ-I",
        "PSUBQ",
        "CVTDQ2PD",
        "PSHUFD",
        "UNPCKLPD",
        "UNPCKHPD",
        "UNPCKLPS",
        "UNPCKHPS",
        "SHUFPS",
        "SHUFPD",
        "PINSRW",
        "PEXTRW",
        "MOVAPS",
        "MOVUPS",
        "MOVAPD",
        "MOVUPD",
        "MOVDQA",
        "MOVDQU",
        "MOVD-X",
        "MOVD-XS",
        "MOVQ-X",
        "MOVQ-XS",
        "MOVQ-XM",
        "PADDQ",
        "PADDD",
        "PADDW",
        "PADDB",
        "PSUBD",
        "PSUBW",
        "PSUBB",
        "PSLLW-I",
        "PSLLD-I",
        "PSLLQ-I",
        "PSRLW-I",
        "PSRLD-I",
        "PSRAW-I",
        "PSRAD-I",
        "MOVD",
        "MOVQ",
        "MOVSS",
        "MOVSS-LD",
        "MOVSD-X",
        "MOVSD-LD",
        "PXOR",
        "PAND",
        "POR",
        "PANDN",
        "XORPS",
        "XORPD",
        "ANDPS",
        "ANDPD",
        "ORPS",
        "ORPD",
        "ANDNPS",
        "ANDNPD",
        "MOV",
        "MOVZX",
        "MOVSX",
        "LEA",
        "PUSH",
        "POP",
        "CALL",
        "JMP",
        "RET",
        "RETI",
        "LEAVE",
        "ADD",
        "SUB",
        "ADC",
        "SBB",
        "INC",
        "DEC",
        "NEG",
        "NOT",
        "AND",
        "OR",
        "XOR",
        "CMP",
        "TEST",
        "SHL",
        "SHR",
        "SAR",
        "SAL",
        "ROL",
        "ROR",
        "BSWAP",
        "BSF",
        "BSR",
        "BT",
        "BTS",
        "BTR",
        "BTC",
        "NOP",
        "NOPL",
        "JO",
        "JNO",
        "JB",
        "JAE",
        "JE",
        "JNE",
        "JBE",
        "JA",
        "JS",
        "JNS",
        "JP",
        "JNP",
        "JL",
        "JGE",
        "JLE",
        "JG",
        "JNB",
        "JZ",
        "JNZ",
        "JNBE",
        "JNL",
        "JNLE",
        "SETZ",
        "SETNZ",
        "CMOVZ",
        "CMOVNZ",
        "SETO",
        "SETNO",
        "SETB",
        "SETNB",
        "SETBE",
        "SETNBE",
        "SETS",
        "SETNS",
        "SETP",
        "SETNP",
        "SETL",
        "SETNL",
        "SETLE",
        "SETNLE",
        "CMOVO",
        "CMOVNO",
        "CMOVB",
        "CMOVNB",
        "CMOVBE",
        "CMOVNBE",
        "CMOVS",
        "CMOVNS",
        "CMOVP",
        "CMOVNP",
        "CMOVL",
        "CMOVNL",
        "CMOVLE",
        "CMOVNLE",
    ];
    if (bank.is_none()
        && !ADMITTED.contains(&mnemonic)
        && !VARIABLE_SHIFTS.contains(&mnemonic)
        && crate::fpu_ops::operation(mnemonic).is_none()
        && crate::sse_arithmetic::operation(mnemonic).is_none()
        && crate::sse_compare::operation(mnemonic).is_none())
        || (d.p.lock
            && ![
                "ADD", "SUB", "AND", "OR", "XOR", "INC", "DEC", "NEG", "NOT", "CMPXCHG", "XADD",
                "XCHG", "BTS", "BTR", "BTC",
            ]
            .contains(&mnemonic))
        || d.p.vex_valid
        || d.p.evex_valid
        || d.p.ad_size
    {
        return Err(format!("unsupported instruction {mnemonic} at {pc:#x}"));
    }
    // Mandatory 66 is consumed by decoder definition selection, rather than
    // retained as the integer operand-size override in PrefixState.
    if VARIABLE_SHIFTS.contains(&mnemonic)
        && !bytes.iter().take_while(|&&b| b != 0x0f).any(|&b| b == 0x66)
    {
        return Err("MMX variable shifts are outside the native XMM profile".into());
    }
    if ["BT", "BTS", "BTR", "BTC"].contains(&mnemonic)
        && (d.p.lock && d.m.is_reg || d.p.ad_size && !d.m.is_reg)
    {
        return Err("bit test requires unlocked registers or 32-bit memory addressing".into());
    }
    if [
        "PUSHF", "POPF", "PUSH", "POP", "CALL", "RET", "RETI", "LEAVE",
    ]
    .contains(&mnemonic)
        && d.p.v_width_d64(XMode::Bits32) != 32
        && !["PUSHF", "POPF"].contains(&mnemonic)
    {
        return Err(format!("16-bit stack operation is not admitted at {pc:#x}"));
    }
    if mnemonic == "POP" && d.op == 0x8f && !d.m.is_reg {
        return Err("POP memory requires post-pop address evaluation support".into());
    }
    if ["DIV", "IDIV"].contains(&mnemonic) && (d.op == 0xf6 || d.p.v_width(XMode::Bits32) != 32) {
        return Err(format!("non-32-bit division not admitted at {pc:#x}"));
    }
    if ["MUL", "IMUL1"].contains(&mnemonic) && (d.op == 0xf6 || d.p.v_width(XMode::Bits32) != 32) {
        return Err(format!(
            "non-32-bit widening multiply not admitted at {pc:#x}"
        ));
    }
    let string = [
        "MOVSB", "MOVSV", "STOSB", "STOSV", "LODSB", "LODSV", "CMPSB", "CMPSV", "SCASB", "SCASV",
    ]
    .contains(&mnemonic);
    if string && (d.p.segment != 0 || d.p.ad_size) {
        return Err(format!(
            "segmented/16-bit-address string operation not admitted at {pc:#x}"
        ));
    }
    let mut inner = Tier0::with_layout(&LAYOUT);
    let shared = Shared {
        guards: RefCell::new(vec![]),
        stored: Cell::new(false),
        last_store: RefCell::new(None),
        scopes: RefCell::new(vec![]),
        next_scope: Cell::new(1),
        stack_store: match mnemonic {
            "CALL" => Some(IlType::U32),
            "PUSHF" => Some(if d.p.op_size {
                IlType::U16
            } else {
                IlType::U32
            }),
            _ => None,
        },
        stack_read: ["POPF", "POP", "RET", "RETI", "LEAVE"].contains(&mnemonic),
        stack_write: ["PUSHF", "PUSH", "CALL"].contains(&mnemonic),
        signed_division: mnemonic == "IDIV",
        aligned_vector: [
            "MOVAPS", "MOVAPD", "MOVDQA", "UNPCKLPD", "UNPCKHPD", "UNPCKLPS", "UNPCKHPS", "SHUFPS",
            "SHUFPD",
        ]
        .contains(&mnemonic)
            || VARIABLE_SHIFTS.contains(&mnemonic),
        segment: match d.p.segment {
            0x26 => Some(0),
            0x2e => Some(1),
            0x36 => Some(2),
            0x3e => Some(3),
            0x64 => Some(4),
            0x65 => Some(5),
            _ => None,
        },
    };
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut bd = Checked {
            inner: &mut inner,
            shared: &shared,
        };
        if mnemonic == "POPF" {
            let sp = bd.reg_read(RegFile(0), 4, IlType::U32);
            let value = bd.mem_read(
                sp,
                if d.p.op_size {
                    IlType::U16
                } else {
                    IlType::U32
                },
            );
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.ldr_x(1, 27, value * 8);
            e.mov_imm64(16, memory::pop_flags as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
        }
        // Generated def84 gives PUSH Ib-sx an 8-bit stack width. The ISA's
        // operand width is 32 here; only the encoded immediate has eight bits.
        if let Some((operation, _)) = bank {
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_r(1, 28);
            e.mov_imm64(2, operation);
            e.mov_imm64(16, crate::stack_bank::execute as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
        } else if ["SAHF", "LAHF"].contains(&mnemonic) {
            // Generated fixed register 4 means SPL, not AH. Bind AH explicitly.
            if mnemonic == "SAHF" {
                let ax = bd.reg_read(RegFile(0), 0, IlType::U32);
                let eight = bd.literal(IlType::U32, 8);
                let ah = bd.shr(ax, eight);
                for bit in [0, 2, 4, 6, 7] {
                    let shift = bd.literal(IlType::U32, bit);
                    let value = bd.shr(ah, shift);
                    let one = bd.literal(IlType::U32, 1);
                    let value = bd.and(value, one);
                    let value = bd.cast(value, IlType::Bool);
                    bd.reg_write(RegFile(1), bit as u32, value);
                }
            } else {
                let mut ah = bd.literal(IlType::U32, 2);
                for bit in [0, 2, 4, 6, 7] {
                    let flag = bd.reg_read(RegFile(1), bit, IlType::Bool);
                    let flag = bd.cast(flag, IlType::U32);
                    let shift = bd.literal(IlType::U32, bit as u128);
                    let value = bd.shl(flag, shift);
                    ah = bd.or(ah, value);
                }
                xfusion_recomp::operand::write_operand(
                    &mut bd,
                    &xfusion_recomp::operand::Operand::Reg {
                        idx: 0,
                        width: 8,
                        high8: true,
                    },
                    ah,
                );
            }
        } else if ["BT", "BTS", "BTR", "BTC"].contains(&mnemonic) {
            use xfusion_recomp::operand::{
                Operand, bind_modrm_reg, bind_modrm_rm, ilty, read_operand, write_operand,
            };
            let width = d.p.v_width(XMode::Bits32);
            let ty = ilty(width);
            let immediate = d.op == 0xba;
            let index = if immediate {
                bd.literal(ty, d.imm0 as u128)
            } else {
                read_operand(&mut bd, &bind_modrm_reg(&d, width))
            };
            let mut dest = bind_modrm_rm(&mut bd, &d, pc as u64, XMode::Bits32, width);
            if !immediate && let Operand::Mem { addr, .. } = dest {
                // A register bit index is signed at operand width. Widen it
                // with sext, not a bit-pattern cast, before floor division.
                let signed = bd.sext(index, IlType::I64);
                let shift = bd.literal(IlType::U8, width.trailing_zeros() as u128);
                let word = bd.shr(signed, shift);
                let shift = bd.literal(IlType::U8, (width / 8).trailing_zeros() as u128);
                let bytes = bd.shl(word, shift);
                let bytes = bd.cast(bytes, IlType::U64);
                let addr = bd.add(addr, bytes);
                dest = Operand::Mem { addr, width };
            }
            let limit = bd.literal(ty, (width - 1) as u128);
            let bit = bd.and(index, limit);
            let one = bd.literal(ty, 1);
            let mask = bd.shl(one, bit);
            let old = if d.p.lock {
                let Operand::Mem { addr, .. } = dest else {
                    unreachable!()
                };
                let (operation, mask) = match mnemonic {
                    "BTS" => (1, mask),
                    "BTR" => (2, bd.not(mask)),
                    "BTC" => (3, mask),
                    _ => unreachable!(),
                };
                bd.mem_rmw_atomic(operation, addr, mask, ty)
            } else {
                read_operand(&mut bd, &dest)
            };
            if mnemonic != "BT" && !d.p.lock {
                let value = match mnemonic {
                    "BTS" => bd.or(old, mask),
                    "BTR" => {
                        let inverse = bd.not(mask);
                        bd.and(old, inverse)
                    }
                    "BTC" => bd.xor(old, mask),
                    _ => unreachable!(),
                };
                write_operand(&mut bd, &dest, value);
            }
            let carry = bd.shr(old, bit);
            let carry = bd.and(carry, one);
            let carry = bd.cast(carry, IlType::Bool);
            bd.reg_write(RegFile(1), 0, carry);
        } else if mnemonic == "RDTSC" {
            let value = bd.inner.literal(IlType::U64, 0);
            let e = &mut bd.inner.enc;
            e.mov_imm64(16, crate::fpu::timestamp as *const () as u64);
            e.put_raw(0xd63f0200);
            e.str_x(0, 27, value * 8);
            let low = bd.cast(value, IlType::U32);
            bd.reg_write(RegFile(0), 0, low);
            let shift = bd.literal(IlType::U64, 32);
            let high = bd.shr(value, shift);
            let high = bd.cast(high, IlType::U32);
            bd.reg_write(RegFile(0), 2, high);
        } else if mnemonic == "FWAIT" {
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_imm64(16, crate::fpu::wait as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
        } else if ["MOVD-X", "MOVD-XS", "MOVQ-X", "MOVQ-XS", "MOVQ-XM"].contains(&mnemonic) {
            use xfusion_recomp::operand::{
                bind_modrm_rm, bind_xmm_reg, bind_xmm_rm, read_operand, write_operand,
            };
            let width = if mnemonic.starts_with("MOVD") { 32 } else { 64 };
            let load = mnemonic == "MOVD-X" || mnemonic == "MOVQ-X";
            if load {
                let src = if width == 32 {
                    bind_modrm_rm(&mut bd, &d, pc as u64, XMode::Bits32, 32)
                } else {
                    bind_xmm_rm(&mut bd, &d, pc as u64, XMode::Bits32, 64)
                };
                let value = read_operand(&mut bd, &src);
                let value = bd.cast(
                    value,
                    if width == 32 {
                        IlType::U32
                    } else {
                        IlType::U64
                    },
                );
                let value = bd.cast(value, IlType::V128);
                write_operand(&mut bd, &bind_xmm_reg(&d, 128), value);
            } else {
                let value = read_operand(&mut bd, &bind_xmm_reg(&d, width));
                let dest = if width == 32 {
                    bind_modrm_rm(&mut bd, &d, pc as u64, XMode::Bits32, 32)
                } else {
                    bind_xmm_rm(
                        &mut bd,
                        &d,
                        pc as u64,
                        XMode::Bits32,
                        if d.m.is_reg { 128 } else { 64 },
                    )
                };
                let value = bd.cast(
                    value,
                    if width == 32 {
                        IlType::U32
                    } else {
                        IlType::U64
                    },
                );
                let value = if width == 64 && d.m.is_reg {
                    bd.cast(value, IlType::V128)
                } else {
                    value
                };
                write_operand(&mut bd, &dest, value);
            }
        } else if let Some(operation) = crate::sse_compare::operation(mnemonic) {
            let address = if d.m.is_reg {
                bd.literal(IlType::U32, 0)
            } else {
                xfusion_recomp::operand::addr_from_modrm(&mut bd, &d, pc as u64, XMode::Bits32)
            };
            let address = bd.address(address, false);
            let description = operation
                | ((d.m.reg as u64 & 7) << 8)
                | ((d.m.rm as u64 & 7) << 12)
                | ((d.m.is_reg as u64) << 16)
                | (((d.imm0 as u64) & 7) << 20);
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_r(1, 28);
            e.mov_imm64(2, description);
            e.ldr_x(3, 27, address * 8);
            e.mov_imm64(16, crate::sse_compare::execute as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
        } else if let Some(operation) = crate::sse_arithmetic::operation(mnemonic) {
            let address = if d.m.is_reg {
                bd.literal(IlType::U32, 0)
            } else {
                xfusion_recomp::operand::addr_from_modrm(&mut bd, &d, pc as u64, XMode::Bits32)
            };
            let address = bd.address(address, false);
            let description = operation
                | ((d.m.reg as u64 & 7) << 8)
                | ((d.m.rm as u64 & 7) << 12)
                | ((d.m.is_reg as u64) << 16);
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_r(1, 28);
            e.mov_imm64(2, description);
            e.ldr_x(3, 27, address * 8);
            e.mov_imm64(16, crate::sse_arithmetic::execute as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
        } else if ["LDMXCSR", "STMXCSR"].contains(&mnemonic) {
            use xfusion_recomp::operand::{bind_modrm_rm, read_operand, write_operand};
            assert!(!d.m.is_reg, "MXCSR requires a memory operand");
            let operand = bind_modrm_rm(&mut bd, &d, pc as u64, XMode::Bits32, 32);
            let input = if mnemonic == "LDMXCSR" {
                read_operand(&mut bd, &operand)
            } else {
                bd.literal(IlType::U32, 0)
            };
            let result = bd.literal(IlType::U32, 0);
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_imm64(1, (result * 8) as u64);
            e.add_r(1, 27, 1);
            e.mov_imm64(2, if mnemonic == "LDMXCSR" { 6 } else { 7 });
            e.ldr_x(3, 27, input * 8);
            e.mov_imm64(16, crate::sse::execute as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
            if mnemonic == "STMXCSR" {
                write_operand(&mut bd, &operand, result);
            }
        } else if let Some((operation, source_width, dest_width)) = crate::sse::conversion(mnemonic)
        {
            use xfusion_recomp::operand::{
                bind_modrm_reg, bind_modrm_rm, bind_xmm_reg, bind_xmm_rm, read_operand,
                write_operand,
            };
            let source = if operation < 4 {
                bind_xmm_rm(&mut bd, &d, pc as u64, XMode::Bits32, source_width)
            } else {
                bind_modrm_rm(&mut bd, &d, pc as u64, XMode::Bits32, source_width)
            };
            let input = read_operand(&mut bd, &source);
            let output = bd.literal(
                if dest_width == 64 {
                    IlType::U64
                } else {
                    IlType::U32
                },
                0,
            );
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_imm64(1, (output * 8) as u64);
            e.add_r(1, 27, 1);
            e.mov_imm64(2, operation);
            e.ldr_x(3, 27, input * 8);
            e.mov_imm64(16, crate::sse::execute as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
            let dest = if operation < 4 {
                bind_modrm_reg(&d, 32)
            } else {
                bind_xmm_reg(&d, dest_width)
            };
            write_operand(&mut bd, &dest, output);
        } else if let Some(operation) = crate::fpu_ops::operation(mnemonic) {
            let address = if !d.m.is_reg && d.op != 0 {
                xfusion_recomp::operand::addr_from_modrm(&mut bd, &d, pc as u64, XMode::Bits32)
            } else {
                bd.literal(IlType::U32, 0)
            };
            let address = bd.address(address, false);
            let e = &mut bd.inner.enc;
            e.ldr_x(0, 28, 90 * 8);
            e.mov_imm64(1, operation | ((d.m.rm as u64 & 7) << 8));
            e.ldr_x(2, 27, address * 8);
            e.mov_r(3, 28);
            e.mov_imm64(16, crate::fpu_ops::execute as *const () as u64);
            e.put_raw(0xd63f0200);
            let at = e.here();
            e.cbz(0, 0);
            shared.guards.borrow_mut().push(at);
        } else if [
            "FNCLEX",
            "FNINIT",
            "FNSTCW",
            "FNSTSW-M",
            "FNSTSW-AX",
            "FLDCW",
        ]
        .contains(&mnemonic)
        {
            use xfusion_recomp::operand::{
                bind_fixed_reg, bind_modrm_rm, read_operand, write_operand,
            };
            let zero = bd.literal(IlType::U64, 0);
            match mnemonic {
                "FNCLEX" => {
                    bd.fp_control(0, zero);
                }
                "FNINIT" => {
                    bd.fp_control(3, zero);
                }
                "FNSTCW" | "FNSTSW-M" => {
                    let op = bind_modrm_rm(&mut bd, &d, pc as u64, XMode::Bits32, 16);
                    let value = bd.fp_control(if mnemonic == "FNSTCW" { 1 } else { 4 }, zero);
                    let value = bd.cast(value, IlType::U16);
                    write_operand(&mut bd, &op, value);
                }
                "FNSTSW-AX" => {
                    let value = bd.fp_control(4, zero);
                    write_operand(&mut bd, &bind_fixed_reg(0, 16), value);
                }
                "FLDCW" => {
                    let op = bind_modrm_rm(&mut bd, &d, pc as u64, XMode::Bits32, 16);
                    let value = read_operand(&mut bd, &op);
                    bd.fp_control(2, value);
                }
                _ => unreachable!(),
            }
        } else if string && (d.p.rep || d.p.rep_nz) {
            // One real iteration per checked entry keeps completed iterations
            // committed on a later page fault, including ECX/ESI/EDI/flags.
            let count = bd.reg_read(RegFile(0), 1, IlType::U32);
            let zero = bd.literal(IlType::U32, 0);
            let empty = bd.eq(count, zero);
            bd.cond(
                empty,
                &mut |bd| {
                    let next = bd.literal(IlType::U32, pc.wrapping_add(d.len as u32) as u128);
                    bd.branch(next, false);
                },
                &mut |bd| {
                    let mut one = d;
                    one.p.rep = false;
                    one.p.rep_nz = false;
                    if ["CMPSB", "CMPSV", "SCASB", "SCASV"].contains(&mnemonic) {
                        bd.compare_string(
                            mnemonic,
                            if mnemonic.ends_with('B') {
                                8
                            } else {
                                d.p.v_width(XMode::Bits32)
                            },
                        );
                    } else {
                        assert!(lift_one(bd, &one, pc as u64, XMode::Bits32, FLAGS_ALL_LIVE));
                    }
                    let one = bd.literal(IlType::U32, 1);
                    let left = bd.sub(count, one);
                    bd.reg_write(RegFile(0), 1, left);
                    let zero = bd.literal(IlType::U32, 0);
                    let mut again = bd.ne(left, zero);
                    if ["CMPSB", "CMPSV", "SCASB", "SCASV"].contains(&mnemonic) {
                        let zf = bd.reg_read(RegFile(1), 6, IlType::Bool);
                        let same = if d.p.rep { zf } else { bd.not(zf) };
                        again = bd.and(again, same);
                    }
                    let repeat = bd.literal(IlType::U32, pc as u128);
                    let next = bd.literal(IlType::U32, pc.wrapping_add(d.len as u32) as u128);
                    let target = bd.ternary(again, repeat, next);
                    bd.branch(target, false);
                },
            );
        } else if mnemonic == "MOV" && (0xa0..=0xa3).contains(&d.op) {
            // Generated moffs operands omit the segment override entirely.
            use xfusion_recomp::operand::{Operand, bind_fixed_reg, read_operand, write_operand};
            let width = if d.op & 1 == 0 {
                8
            } else {
                d.p.v_width(XMode::Bits32)
            };
            let mut addr = bd.literal(IlType::U64, d.imm0 as u32 as u128);
            if let Some(si) = shared.segment {
                let base = bd.reg_read(RegFile(2), si, IlType::U64);
                addr = bd.add(base, addr);
            }
            let memory = Operand::Mem { addr, width };
            let register = bind_fixed_reg(0, width);
            let (dst, src) = if d.op & 2 == 0 {
                (register, memory)
            } else {
                (memory, register)
            };
            let value = read_operand(&mut bd, &src);
            write_operand(&mut bd, &dst, value);
        } else if mnemonic == "LEA" {
            // LEA never applies segment bases, even with an FS override.
            let mut plain = d;
            plain.p.segment = 0;
            if !lift_one(&mut bd, &plain, pc as u64, XMode::Bits32, FLAGS_ALL_LIVE) {
                return Err(String::from("LEA not lifted"));
            }
        } else if mnemonic == "PUSH" && d.op == 0x6a {
            let sp = bd.reg_read(RegFile(0), 4, IlType::U32);
            let four = bd.literal(IlType::U32, 4);
            let next_sp = bd.sub(sp, four);
            let value = bd.literal(IlType::U32, d.imm0 as i8 as i32 as u32 as u128);
            bd.mem_write(next_sp, value);
            bd.reg_write(RegFile(0), 4, next_sp);
        } else if ["CMPSB", "CMPSV", "SCASB", "SCASV"].contains(&mnemonic) {
            bd.compare_string(
                mnemonic,
                if mnemonic.ends_with('B') {
                    8
                } else {
                    d.p.v_width(XMode::Bits32)
                },
            );
        } else if !lift_one(
            &mut bd,
            &d,
            pc as u64,
            XMode::Bits32,
            FLAGS_ALL_LIVE
                | if mnemonic == "POPF" && !d.p.op_size {
                    1 << 21
                } else {
                    0
                },
        ) {
            return Err(String::from("instruction not lifted"));
        }
        if !bd.branched() {
            let target = bd.literal(IlType::U32, pc.wrapping_add(d.len as u32) as u128);
            bd.branch(target, false);
        }
        Ok(())
    }));
    match outcome {
        Ok(result) => result?,
        Err(error) => {
            let why = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap_or("unknown compiler panic");
            return Err(format!("unsupported lowering {mnemonic} at {pc:#x}: {why}"));
        }
    }
    let end = inner.enc.here();
    for &at in shared.guards.borrow().iter() {
        let offset = end
            .checked_sub(at)
            .filter(|x| *x < (1 << 18))
            .ok_or("guard branch exceeds range")?;
        inner.enc.patch(at, 0xb4000000 | ((offset as u32) << 5)); // cbz x0,epilogue
    }
    inner.enc.ldr_x(27, 31, 0);
    inner.enc.ldr_x(28, 31, 8);
    inner.enc.ldr_x(30, 31, 16);
    inner.enc.add_i(31, 31, 32);
    inner.enc.ret();
    Ok(Instruction {
        code: Code::new(&inner.enc)?,
        length: d.len as usize,
    })
}

#[cfg(test)]
mod forwarding_tests {
    use super::*;
    #[test]
    fn forwarding_rejects_other_addresses_widths_and_nondominating_stores() {
        for shape in 0..7 {
            let result = std::panic::catch_unwind(|| {
                let mut inner = Tier0::with_layout(&LAYOUT);
                let shared = Shared {
                    guards: RefCell::new(vec![]),
                    stored: Cell::new(false),
                    last_store: RefCell::new(None),
                    scopes: RefCell::new(vec![]),
                    next_scope: Cell::new(1),
                    stack_store: None,
                    stack_read: false,
                    stack_write: false,
                    segment: None,
                    signed_division: false,
                    aligned_vector: false,
                };
                let mut b = Checked {
                    inner: &mut inner,
                    shared: &shared,
                };
                let address = b.literal(IlType::U32, 0x20000);
                let other = b.literal(IlType::U32, 0x30000);
                let value = b.literal(IlType::U32, 0x12345678);
                let condition = b.literal(IlType::Bool, 1);
                if shape <= 2 || shape == 6 {
                    b.mem_write(address, value);
                    match shape {
                        1 => {
                            b.mem_read(other, IlType::U32);
                        }
                        2 => {
                            b.mem_read(address, IlType::U16);
                        }
                        6 => b.mem_write(address, value),
                        _ => {
                            b.mem_read(address, IlType::U32);
                        }
                    }
                } else {
                    b.cond(
                        condition,
                        &mut |b| {
                            b.mem_write(address, value);
                            if shape == 5 {
                                b.mem_read(address, IlType::U32);
                            }
                        },
                        &mut |b| {
                            if shape == 4 {
                                b.mem_read(address, IlType::U32);
                            }
                        },
                    );
                    if shape == 3 {
                        b.mem_read(address, IlType::U32);
                    }
                }
            });
            assert_eq!(result.is_ok(), shape == 0 || shape == 5, "shape {shape}");
        }
    }
}
