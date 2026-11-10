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
        // SIMD/FP state is not admitted by this initial scalar runner.
        _ => panic!("unadmitted register file/index: {file:?}/{idx}"),
    },
    off_rr: 0,
    gpr_w_zext: true,
};
struct Shared {
    guards: RefCell<Vec<usize>>,
    stored: Cell<bool>,
    call: bool,
    stack_read: bool,
    stack_write: bool,
    segment: Option<u32>,
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
        _ => panic!("unadmitted memory type {ty:?}"),
    }
}
impl Checked<'_> {
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
    fn checked_val(&self, v: u32) -> u32 {
        assert!(v < 4096, "spill limit");
        v
    }
}
impl Builder for Checked<'_> {
    type Val = u32;
    fn ty_of(&self, v: u32) -> IlType {
        self.inner.ty_of(v)
    }
    fn mem_read(&mut self, addr: u32, ty: IlType) -> u32 {
        assert!(
            !self.shared.stored.get(),
            "read after committed store requires precise partial-state support"
        );
        let addr = self.address(addr, self.shared.stack_read);
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
        let value = if self.shared.call {
            self.inner.cast(value, IlType::U32)
        } else {
            value
        };
        let addr = self.address(addr, self.shared.stack_write);
        self.guard(addr, width(self.inner.ty_of(value)), true);
        self.inner.mem_write(addr, value);
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
        yes(self);
        let jump = self.inner.enc.here();
        self.inner.enc.b(0);
        let otherwise = self.inner.enc.here();
        no(self);
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
    #[allow(unused_variables)]
    fn div(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: div")
    }
    #[allow(unused_variables)]
    fn rem(&mut self, a: Self::Val, b: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: rem")
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
        panic!("unsupported JIT operation: vzip")
    }
    #[allow(unused_variables)]
    fn vfbin(&mut self, a: Self::Val, b: Self::Val, elem_bits: u32, op: u32) -> Self::Val {
        panic!("unsupported JIT operation: vfbin")
    }
    #[allow(unused_variables)]
    fn vibin(&mut self, a: Self::Val, b: Self::Val, elem_bits: u32, op: u32) -> Self::Val {
        panic!("unsupported JIT operation: vibin")
    }
    #[allow(unused_variables)]
    fn vishi(&mut self, a: Self::Val, ew: u32, count: u32, dir: u32) -> Self::Val {
        panic!("unsupported JIT operation: vishi")
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
        panic!("unsupported JIT operation: vshuf")
    }
    #[allow(unused_variables)]
    fn vshufw(&mut self, src: Self::Val, sel: u32, hi: bool) -> Self::Val {
        panic!("unsupported JIT operation: vshufw")
    }
    #[allow(unused_variables)]
    fn vcvt(&mut self, a: Self::Val, kind: u32) -> Self::Val {
        panic!("unsupported JIT operation: vcvt")
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
        panic!("unsupported JIT operation: velement_read")
    }
    #[allow(unused_variables)]
    fn velement_write(&mut self, v: Self::Val, i: Self::Val, e: Self::Val) -> Self::Val {
        panic!("unsupported JIT operation: velement_write")
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
        panic!("unsupported JIT operation: mem_rmw_atomic")
    }
    #[allow(unused_variables)]
    fn fence(&mut self) {
        panic!("unsupported JIT operation: fence")
    }
    #[allow(unused_variables)]
    fn mem_cas_atomic(
        &mut self,
        _addr: Self::Val,
        _expected: Self::Val,
        _new: Self::Val,
        _ty: IlType,
    ) -> Self::Val {
        panic!("unsupported JIT operation: mem_cas_atomic")
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
        panic!("unsupported JIT operation: call_intrinsic")
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
    let d =
        decode_insn(bytes, XMode::Bits32).ok_or_else(|| format!("decode failure at {pc:#x}"))?;
    let mnemonic = DEF_MNEMONICS[d.def_id as usize];
    // String loops, FP/SIMD, interrupts, system instructions and implicit memory
    // operations are outside the initial admitted subset. Never execute an empty
    // generated template as if it implemented a guest operation.
    const ADMITTED: &[&str] = &[
        "MOV", "MOVZX", "MOVSX", "LEA", "PUSH", "POP", "CALL", "JMP", "RET", "RETI", "LEAVE",
        "ADD", "SUB", "ADC", "SBB", "INC", "DEC", "NEG", "NOT", "AND", "OR", "XOR", "CMP", "TEST",
        "SHL", "SHR", "SAR", "SAL", "ROL", "ROR", "BSWAP", "NOP", "NOPL", "JO", "JNO", "JB", "JAE",
        "JE", "JNE", "JBE", "JA", "JS", "JNS", "JP", "JNP", "JL", "JGE", "JLE", "JG", "JNB", "JZ",
        "JNZ", "JNBE", "JNL", "JNLE", "SETZ", "SETNZ", "CMOVZ", "CMOVNZ", "SETO", "SETNO", "SETB",
        "SETNB", "SETBE", "SETNBE", "SETS", "SETNS", "SETP", "SETNP", "SETL", "SETNL", "SETLE",
        "SETNLE", "CMOVO", "CMOVNO", "CMOVB", "CMOVNB", "CMOVBE", "CMOVNBE", "CMOVS", "CMOVNS",
        "CMOVP", "CMOVNP", "CMOVL", "CMOVNL", "CMOVLE", "CMOVNLE",
    ];
    if !ADMITTED.contains(&mnemonic) || d.p.lock || d.p.ad_size {
        return Err(format!("unsupported instruction {mnemonic} at {pc:#x}"));
    }
    if ["PUSH", "POP", "CALL", "RET", "RETI", "LEAVE"].contains(&mnemonic)
        && d.p.v_width_d64(XMode::Bits32) != 32
    {
        return Err(format!("16-bit stack operation is not admitted at {pc:#x}"));
    }
    if mnemonic == "POP" && d.op == 0x8f && !d.m.is_reg {
        return Err("POP memory requires post-pop address evaluation support".into());
    }
    let mut inner = Tier0::with_layout(&LAYOUT);
    let shared = Shared {
        guards: RefCell::new(vec![]),
        stored: Cell::new(false),
        call: mnemonic == "CALL",
        stack_read: ["POP", "RET", "RETI", "LEAVE"].contains(&mnemonic),
        stack_write: ["PUSH", "CALL"].contains(&mnemonic),
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
        // Generated def84 gives PUSH Ib-sx an 8-bit stack width. The ISA's
        // operand width is 32 here; only the encoded immediate has eight bits.
        if mnemonic == "MOV" && (0xa0..=0xa3).contains(&d.op) {
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
        } else if !lift_one(&mut bd, &d, pc as u64, XMode::Bits32, FLAGS_ALL_LIVE) {
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
