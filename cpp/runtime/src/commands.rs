//! Explicit buffer-copy command allocator/list subset. No drawing/shader stubs.
use super::*;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
const E_FAIL: i32 = 0x80004005u32 as i32;
const ALLOCATOR: Guid = guid!(
    0x6102dee4, 0xaf59, 0x4b09, 0xb9, 0x99, 0xb4, 0x4d, 0x73, 0xf0, 0x9b, 0x24
);
const LIST: Guid = guid!(
    0x7116d91c, 0xe7e4, 0x47ce, 0xb8, 0xc6, 0xec, 0x81, 0x68, 0xf4, 0x37, 0xe5
);
const GRAPHICS: Guid = guid!(
    0x5b160d0f, 0xac1b, 0x4185, 0x8b, 0xa8, 0xb3, 0xae, 0x42, 0xa5, 0xa4, 0x55
);
#[derive(Default)]
struct Allocation {
    recording: bool,
    epoch: u64,
    in_flight: usize,
}
#[repr(C)]
struct Allocator {
    vtable: *const AllocatorVtable,
    refs: AtomicU32,
    metadata: Metadata,
    kind: i32,
    state: Arc<Mutex<Allocation>>,
    parent: Parent,
}
#[derive(Clone)]
struct Copy {
    source: metal::Buffer,
    destination: metal::Buffer,
    source_offset: u64,
    destination_offset: u64,
    length: u64,
    source_usage: Option<Arc<Mutex<BufferUsage>>>,
    destination_usage: Option<Arc<Mutex<BufferUsage>>>,
}
#[derive(Clone)]
struct Transition {
    usage: Arc<Mutex<BufferUsage>>,
    before: u32,
    after: u32,
}
#[derive(Clone)]
enum Command {
    Copy(Copy),
    Transition(Transition),
}
struct Recording {
    allocation: Arc<Mutex<Allocation>>,
    epoch: u64,
    open: bool,
    error: Option<i32>,
    commands: Vec<Command>,
    executing: Arc<AtomicBool>,
}
impl Drop for Recording {
    fn drop(&mut self) {
        if self.open {
            self.allocation.lock().unwrap().recording = false;
        }
    }
}
#[repr(C)]
struct List {
    vtable: *const ListVtable,
    refs: AtomicU32,
    metadata: Metadata,
    kind: i32,
    recording: Mutex<Recording>,
    parent: Parent,
}
// The immutable native buffers and allocator epoch outlive COM caller handles.
// Completion releases the execution reservation before later fence events fire.
pub(super) struct Submission {
    commands: Vec<Command>,
    allocation: Arc<Mutex<Allocation>>,
    executing: Arc<AtomicBool>,
    finished: AtomicBool,
    access: Option<Arc<states::Access>>,
}
impl Submission {
    pub(super) fn commit(&self) {
        if let Some(access) = &self.access {
            access.commit();
        }
    }
    pub(super) fn complete(&self) {
        self.finish(false);
    }
    pub(super) fn finish(&self, failed: bool) {
        if !self.finished.swap(true, Ordering::AcqRel) {
            if let Some(access) = &self.access {
                access.complete_member(failed);
            }
            self.allocation.lock().unwrap().in_flight -= 1;
            self.executing.store(false, Ordering::Release);
        }
    }
    pub(super) fn encode(&self, encoder: &metal::BlitCommandEncoderRef) {
        for command in &self.commands {
            // Metal tracks hazards for these serial blits. Transition entries
            // validate the D3D state contract; they do not need a second encoder.
            if let Command::Copy(c) = command
                && c.length != 0
            {
                encoder.copy_from_buffer(
                    &c.source,
                    c.source_offset,
                    &c.destination,
                    c.destination_offset,
                    c.length,
                );
            }
        }
    }
}
impl Drop for Submission {
    fn drop(&mut self) {
        self.complete();
    }
}
macro_rules! ownership {
    ($ty:ty,[$($id:expr),+],$q:ident,$a:ident,$r:ident,$name:ident,$device:ident) => {
        unsafe extern "C" fn $q(this:*mut $ty,iid:*const Guid,out:*mut *mut c_void)->i32 {
            if let Err(e)=unsafe{clear(out)} {return e;} if iid.is_null(){return E_POINTER;}
            if ![IID_IUNKNOWN,IID_ID3D12OBJECT,CHILD,$($id),+].contains(unsafe{&*iid}) {return E_NOINTERFACE;}
            unsafe{$a(this);*out=this.cast();} S_OK
        }
        unsafe extern "C" fn $a(this:*mut $ty)->u32 {increment(unsafe{&(*this).refs})}
        unsafe extern "C" fn $r(this:*mut $ty)->u32 {
            let n=unsafe{(*this).refs.fetch_sub(1,Ordering::Release)};
            if n==1 {fence(Ordering::Acquire);unsafe{drop(Box::from_raw(this));}0}else{n-1}
        }
        unsafe extern "C" fn $name(this:*mut $ty,name:*const u32)->i32 {let m=unsafe{(*this).metadata.0};unsafe{((*(*m).vtable).set_name)(m,name)}}
        unsafe extern "C" fn $device(this:*mut $ty,iid:*const Guid,out:*mut *mut c_void)->i32 {unsafe{device_query((*this).parent.0,iid,out)}}
    }
}
ownership!(
    Allocator,
    [PAGEABLE, ALLOCATOR],
    allocator_query,
    allocator_add,
    allocator_release,
    allocator_name,
    allocator_device
);
ownership!(
    List,
    [LIST, GRAPHICS],
    list_query,
    list_add,
    list_release,
    list_name,
    list_device
);
metadata_methods!(Allocator, allocator_set, allocator_interface, allocator_get);
metadata_methods!(List, list_set, list_interface, list_get);
unsafe extern "C" fn allocator_reset(this: *mut Allocator) -> i32 {
    let mut state = unsafe { (*this).state.lock().unwrap() };
    if state.recording || state.in_flight != 0 {
        return E_FAIL;
    }
    let Some(next) = state.epoch.checked_add(1) else {
        return E_OUTOFMEMORY;
    };
    state.epoch = next;
    S_OK
}
unsafe extern "C" fn kind(this: *mut List) -> i32 {
    unsafe { (*this).kind }
}
unsafe extern "C" fn close(this: *mut List) -> i32 {
    let mut r = unsafe { (*this).recording.lock().unwrap() };
    if let Some(error) = r.error {
        if r.open {
            r.open = false;
            r.allocation.lock().unwrap().recording = false;
        }
        return error;
    }
    if !r.open {
        r.error = Some(E_FAIL);
        return E_FAIL;
    }
    r.open = false;
    r.allocation.lock().unwrap().recording = false;
    S_OK
}
unsafe fn begin(
    allocator: *mut Allocator,
    device: *mut Device,
    kind: i32,
) -> Result<Recording, i32> {
    if allocator.is_null() {
        return Err(E_POINTER);
    }
    if unsafe { (*allocator).vtable } != &ALLOCATOR_VTABLE
        || unsafe { (*allocator).parent.0 } != device
        || unsafe { (*allocator).kind } != kind
    {
        return Err(E_INVALIDARG);
    }
    let allocation = unsafe { Arc::clone(&(*allocator).state) };
    let epoch = {
        let mut a = allocation.lock().unwrap();
        if a.recording {
            return Err(E_INVALIDARG);
        }
        a.recording = true;
        a.epoch
    };
    Ok(Recording {
        allocation,
        epoch,
        open: true,
        error: None,
        commands: Vec::new(),
        executing: Arc::new(AtomicBool::new(false)),
    })
}
unsafe extern "C" fn reset(this: *mut List, allocator: *mut Allocator, pipeline: usize) -> i32 {
    let mut r = unsafe { (*this).recording.lock().unwrap() };
    if let Some(error) = r.error {
        return error;
    }
    if r.open {
        return E_FAIL;
    }
    if pipeline != 0 {
        return E_NOTIMPL;
    }
    match unsafe { begin(allocator, (*this).parent.0, (*this).kind) } {
        Ok(recording) => {
            *r = recording;
            S_OK
        }
        Err(e) => e,
    }
}
fn extent(offset: u64, length: u64, width: u64) -> Result<(), i32> {
    if offset.checked_add(length).is_none_or(|end| end > width) {
        return Err(E_INVALIDARG);
    }
    // This first native-blit path accepts aligned copies only. Byte-granular
    // lowering remains explicit unsupported, never rounded/truncated/padded.
    if !offset.is_multiple_of(4) || !length.is_multiple_of(4) {
        return Err(E_NOTIMPL);
    }
    Ok(())
}
unsafe fn copy_command(
    device: *mut Device,
    dst: *mut Resource,
    dst_offset: u64,
    src: *mut Resource,
    src_offset: u64,
    length: Option<u64>,
) -> Result<Copy, i32> {
    if dst.is_null() || src.is_null() {
        return Err(E_POINTER);
    }
    if unsafe { (*dst).vtable } != &RESOURCE_VTABLE || unsafe { (*src).vtable } != &RESOURCE_VTABLE
    {
        return Err(E_INVALIDARG);
    }
    let (d, s) = unsafe { (&*dst, &*src) };
    if d.parent.0 != device || s.parent.0 != device {
        return Err(E_INVALIDARG);
    }
    if ![1, 3].contains(&d.heap.kind)
        || ![1, 2].contains(&s.heap.kind)
        || Arc::ptr_eq(&d.usage, &s.usage)
    {
        return Err(E_NOTIMPL);
    }
    let length = match length {
        Some(n) => n,
        None => {
            if d.desc.width != s.desc.width {
                return Err(E_INVALIDARG);
            }
            s.desc.width
        }
    };
    if length == 0 && (d.heap.kind == 1 || s.heap.kind == 1) {
        return Err(E_NOTIMPL);
    }
    extent(dst_offset, length, d.desc.width)?;
    extent(src_offset, length, s.desc.width)?;
    Ok(Copy {
        source: s.buffer.to_owned(),
        destination: d.buffer.to_owned(),
        source_offset: s.offset as u64 + src_offset,
        destination_offset: d.offset as u64 + dst_offset,
        length,
        source_usage: (s.heap.kind == 1).then(|| Arc::clone(&s.usage)),
        destination_usage: (d.heap.kind == 1).then(|| Arc::clone(&d.usage)),
    })
}
unsafe fn record_copy(
    this: *mut List,
    dst: *mut Resource,
    dst_offset: u64,
    src: *mut Resource,
    src_offset: u64,
    length: Option<u64>,
) {
    let mut r = unsafe { (*this).recording.lock().unwrap() };
    if r.error.is_some() {
        return;
    }
    if !r.open {
        r.error = Some(E_FAIL);
        return;
    }
    let command =
        unsafe { copy_command((*this).parent.0, dst, dst_offset, src, src_offset, length) };
    match command {
        Ok(command) => {
            if r.commands.try_reserve(1).is_err() {
                r.error = Some(E_OUTOFMEMORY);
            } else {
                r.commands.push(Command::Copy(command));
            }
        }
        Err(e) => r.error = Some(e),
    }
}
unsafe extern "C" fn copy_buffer(
    this: *mut List,
    dst: *mut Resource,
    dst_offset: u64,
    src: *mut Resource,
    src_offset: u64,
    length: u64,
) {
    unsafe {
        record_copy(this, dst, dst_offset, src, src_offset, Some(length));
    }
}
unsafe extern "C" fn copy_resource(this: *mut List, dst: *mut Resource, src: *mut Resource) {
    unsafe {
        record_copy(this, dst, 0, src, 0, None);
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Barrier {
    kind: i32,
    flags: i32,
    resource: *mut Resource,
    subresource: u32,
    before: u32,
    after: u32,
}
unsafe fn transition(device: *mut Device, kind: i32, b: Barrier) -> Result<Transition, i32> {
    if kind != 0 || b.kind != 0 || b.flags != 0 {
        return Err(E_NOTIMPL);
    }
    if b.resource.is_null() {
        return Err(E_POINTER);
    }
    if unsafe { (*b.resource).vtable } != &RESOURCE_VTABLE {
        return Err(E_INVALIDARG);
    }
    let resource = unsafe { &*b.resource };
    if resource.parent.0 != device
        || resource.heap.kind != 1
        || ![0, u32::MAX].contains(&b.subresource)
    {
        return Err(E_INVALIDARG);
    }
    if !valid_default_state(b.before) || !valid_default_state(b.after) {
        return Err(E_NOTIMPL);
    }
    if b.before == b.after {
        return Err(E_NOTIMPL);
    }
    Ok(Transition {
        usage: Arc::clone(&resource.usage),
        before: b.before,
        after: b.after,
    })
}
unsafe extern "C" fn barriers(this: *mut List, count: u32, data: usize) {
    if count == 0 {
        unsupported_void("zero-count resource barrier");
    }
    let mut r = unsafe { (*this).recording.lock().unwrap() };
    if r.error.is_some() {
        return;
    }
    if !r.open {
        r.error = Some(E_FAIL);
        return;
    }
    if data == 0 {
        r.error = Some(E_POINTER);
        return;
    }
    if r.commands.try_reserve(count as usize).is_err() {
        r.error = Some(E_OUTOFMEMORY);
        return;
    }
    for &b in unsafe { std::slice::from_raw_parts(data as *const Barrier, count as usize) } {
        match unsafe { transition((*this).parent.0, (*this).kind, b) } {
            Ok(t) => r.commands.push(Command::Transition(t)),
            Err(e) => {
                r.error = Some(e);
                return;
            }
        }
    }
}
// No queue state/native work is mutated until every list has been validated.
// Reserving an execution is rolled back by Submission::drop on any later error.
pub(super) unsafe fn prepare(
    device: *mut Device,
    kind: i32,
    count: u32,
    lists: usize,
    owner: &Arc<()>,
) -> Result<Vec<Arc<Submission>>, i32> {
    if count == 0 {
        return Err(E_NOTIMPL);
    }
    if lists == 0 {
        return Err(E_POINTER);
    }
    let mut batches = Vec::new();
    batches
        .try_reserve(count as usize)
        .map_err(|_| E_OUTOFMEMORY)?;
    for &list in unsafe { std::slice::from_raw_parts(lists as *const *mut List, count as usize) } {
        if list.is_null() || unsafe { (*list).vtable } != &LIST_VTABLE {
            return Err(E_INVALIDARG);
        }
        if unsafe { (*list).parent.0 } != device || unsafe { (*list).kind } != kind {
            return Err(E_INVALIDARG);
        }
        let r = unsafe { (*list).recording.lock().unwrap() };
        if r.open || r.error.is_some() {
            return Err(E_FAIL);
        }
        let mut a = r.allocation.lock().unwrap();
        if a.epoch != r.epoch {
            return Err(E_FAIL);
        }
        let Some(next) = a.in_flight.checked_add(1) else {
            return Err(E_OUTOFMEMORY);
        };
        let mut commands = Vec::new();
        commands
            .try_reserve(r.commands.len())
            .map_err(|_| E_OUTOFMEMORY)?;
        commands.extend(r.commands.iter().cloned());
        if r.executing.swap(true, Ordering::AcqRel) {
            return Err(E_FAIL);
        }
        a.in_flight = next;
        batches.push(Arc::new(Submission {
            commands,
            allocation: Arc::clone(&r.allocation),
            executing: Arc::clone(&r.executing),
            finished: AtomicBool::new(false),
            access: None,
        }));
    }
    if let Some(access) = states::reserve(&batches, owner)? {
        for batch in &mut batches {
            Arc::get_mut(batch).unwrap().access = Some(Arc::clone(&access));
        }
    }
    Ok(batches)
}
#[path = "buffer_states.rs"]
mod states;
#[repr(C)]
struct AllocatorVtable {
    query: unsafe extern "C" fn(*mut Allocator, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Allocator) -> u32,
    release: unsafe extern "C" fn(*mut Allocator) -> u32,
    get: unsafe extern "C" fn(*mut Allocator, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Allocator, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Allocator, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Allocator, *const u32) -> i32,
    device: unsafe extern "C" fn(*mut Allocator, *const Guid, *mut *mut c_void) -> i32,
    reset: unsafe extern "C" fn(*mut Allocator) -> i32,
}
static ALLOCATOR_VTABLE: AllocatorVtable = AllocatorVtable {
    query: allocator_query,
    add: allocator_add,
    release: allocator_release,
    get: allocator_get,
    set: allocator_set,
    interface: allocator_interface,
    name: allocator_name,
    device: allocator_device,
    reset: allocator_reset,
};
pub(super) unsafe extern "C" fn create_allocator(
    device: *mut Device,
    kind: i32,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if ![0, 2, 3].contains(&kind) {
        return E_NOTIMPL;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, ALLOCATOR].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    unsafe {
        device_add(device);
        *out = Box::into_raw(Box::new(Allocator {
            vtable: &ALLOCATOR_VTABLE,
            refs: AtomicU32::new(1),
            metadata: Metadata::new(),
            kind,
            state: Arc::new(Mutex::new(Allocation::default())),
            parent: Parent(device),
        }))
        .cast();
    }
    S_OK
}
pub(super) unsafe extern "C" fn create_list(
    device: *mut Device,
    node: u32,
    kind: i32,
    allocator: usize,
    pipeline: usize,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if node > 1 || pipeline != 0 || ![0, 2, 3].contains(&kind) {
        return E_NOTIMPL;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, LIST, GRAPHICS].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    let recording = match unsafe { begin(allocator as *mut Allocator, device, kind) } {
        Ok(r) => r,
        Err(e) => return e,
    };
    unsafe {
        device_add(device);
        *out = Box::into_raw(Box::new(List {
            vtable: &LIST_VTABLE,
            refs: AtomicU32::new(1),
            metadata: Metadata::new(),
            kind,
            recording: Mutex::new(recording),
            parent: Parent(device),
        }))
        .cast();
    }
    S_OK
}

// All 60 ID3D12GraphicsCommandList0 slots, in pinned DirectX-Headers order.
// Opaque unsupported pointer arguments use source-ABI pointer-width integers;
// by-value descriptor handles and floating-point arguments retain their ABI.
#[repr(C)]
#[derive(Clone, Copy)]
struct CpuHandle {
    ptr: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct GpuHandle {
    ptr: u64,
}
macro_rules! unsupported_methods {
    ($($name:ident($($arg:ty),*);)*) => {$(
        unsafe extern "C" fn $name(_: *mut List,$(_: $arg),*) {unsupported_void(concat!("command list ",stringify!($name)));}
    )*};
}
unsupported_methods! {
ClearState(usize);
DrawInstanced(u32,u32,u32,u32);
DrawIndexedInstanced(u32,u32,u32,i32,u32);
Dispatch(u32,u32,u32);
CopyTextureRegion(usize,u32,u32,u32,usize,usize);
CopyTiles(usize,usize,usize,usize,u64,i32);
ResolveSubresource(usize,u32,usize,u32,i32);
IASetPrimitiveTopology(i32);
RSSetViewports(u32,usize);
RSSetScissorRects(u32,usize);
OMSetBlendFactor(usize);
OMSetStencilRef(u32);
SetPipelineState(usize);

ExecuteBundle(usize);
SetDescriptorHeaps(u32,usize);
SetComputeRootSignature(usize);
SetGraphicsRootSignature(usize);
SetComputeRootDescriptorTable(u32,GpuHandle);
SetGraphicsRootDescriptorTable(u32,GpuHandle);
SetComputeRoot32BitConstant(u32,u32,u32);
SetGraphicsRoot32BitConstant(u32,u32,u32);
SetComputeRoot32BitConstants(u32,u32,usize,u32);
SetGraphicsRoot32BitConstants(u32,u32,usize,u32);
SetComputeRootConstantBufferView(u32,u64);
SetGraphicsRootConstantBufferView(u32,u64);
SetComputeRootShaderResourceView(u32,u64);
SetGraphicsRootShaderResourceView(u32,u64);
SetComputeRootUnorderedAccessView(u32,u64);
SetGraphicsRootUnorderedAccessView(u32,u64);
IASetIndexBuffer(usize);
IASetVertexBuffers(u32,u32,usize);
SOSetTargets(u32,u32,usize);
OMSetRenderTargets(u32,usize,i32,usize);
ClearDepthStencilView(CpuHandle,i32,f32,u8,u32,usize);
ClearRenderTargetView(CpuHandle,usize,u32,usize);
ClearUnorderedAccessViewUint(GpuHandle,CpuHandle,usize,usize,u32,usize);
ClearUnorderedAccessViewFloat(GpuHandle,CpuHandle,usize,usize,u32,usize);
DiscardResource(usize,usize);
BeginQuery(usize,i32,u32);
EndQuery(usize,i32,u32);
ResolveQueryData(usize,i32,u32,u32,usize,u64);
SetPredication(usize,u64,i32);
SetMarker(u32,usize,u32);
BeginEvent(u32,usize,u32);
EndEvent();
ExecuteIndirect(usize,u32,usize,u64,usize,u64);
}
#[repr(C)]
struct ListVtable {
    QueryInterface: unsafe extern "C" fn(*mut List, *const Guid, *mut *mut c_void) -> i32,
    AddRef: unsafe extern "C" fn(*mut List) -> u32,
    Release: unsafe extern "C" fn(*mut List) -> u32,
    GetPrivateData: unsafe extern "C" fn(*mut List, *const Guid, *mut u32, *mut c_void) -> i32,
    SetPrivateData: unsafe extern "C" fn(*mut List, *const Guid, u32, *const c_void) -> i32,
    SetPrivateDataInterface: unsafe extern "C" fn(*mut List, *const Guid, *mut c_void) -> i32,
    SetName: unsafe extern "C" fn(*mut List, *const u32) -> i32,
    GetDevice: unsafe extern "C" fn(*mut List, *const Guid, *mut *mut c_void) -> i32,
    GetType: unsafe extern "C" fn(*mut List) -> i32,
    Close: unsafe extern "C" fn(*mut List) -> i32,
    Reset: unsafe extern "C" fn(*mut List, *mut Allocator, usize) -> i32,
    ClearState: unsafe extern "C" fn(*mut List, usize),
    DrawInstanced: unsafe extern "C" fn(*mut List, u32, u32, u32, u32),
    DrawIndexedInstanced: unsafe extern "C" fn(*mut List, u32, u32, u32, i32, u32),
    Dispatch: unsafe extern "C" fn(*mut List, u32, u32, u32),
    CopyBufferRegion: unsafe extern "C" fn(*mut List, *mut Resource, u64, *mut Resource, u64, u64),
    CopyTextureRegion: unsafe extern "C" fn(*mut List, usize, u32, u32, u32, usize, usize),
    CopyResource: unsafe extern "C" fn(*mut List, *mut Resource, *mut Resource),
    CopyTiles: unsafe extern "C" fn(*mut List, usize, usize, usize, usize, u64, i32),
    ResolveSubresource: unsafe extern "C" fn(*mut List, usize, u32, usize, u32, i32),
    IASetPrimitiveTopology: unsafe extern "C" fn(*mut List, i32),
    RSSetViewports: unsafe extern "C" fn(*mut List, u32, usize),
    RSSetScissorRects: unsafe extern "C" fn(*mut List, u32, usize),
    OMSetBlendFactor: unsafe extern "C" fn(*mut List, usize),
    OMSetStencilRef: unsafe extern "C" fn(*mut List, u32),
    SetPipelineState: unsafe extern "C" fn(*mut List, usize),
    ResourceBarrier: unsafe extern "C" fn(*mut List, u32, usize),
    ExecuteBundle: unsafe extern "C" fn(*mut List, usize),
    SetDescriptorHeaps: unsafe extern "C" fn(*mut List, u32, usize),
    SetComputeRootSignature: unsafe extern "C" fn(*mut List, usize),
    SetGraphicsRootSignature: unsafe extern "C" fn(*mut List, usize),
    SetComputeRootDescriptorTable: unsafe extern "C" fn(*mut List, u32, GpuHandle),
    SetGraphicsRootDescriptorTable: unsafe extern "C" fn(*mut List, u32, GpuHandle),
    SetComputeRoot32BitConstant: unsafe extern "C" fn(*mut List, u32, u32, u32),
    SetGraphicsRoot32BitConstant: unsafe extern "C" fn(*mut List, u32, u32, u32),
    SetComputeRoot32BitConstants: unsafe extern "C" fn(*mut List, u32, u32, usize, u32),
    SetGraphicsRoot32BitConstants: unsafe extern "C" fn(*mut List, u32, u32, usize, u32),
    SetComputeRootConstantBufferView: unsafe extern "C" fn(*mut List, u32, u64),
    SetGraphicsRootConstantBufferView: unsafe extern "C" fn(*mut List, u32, u64),
    SetComputeRootShaderResourceView: unsafe extern "C" fn(*mut List, u32, u64),
    SetGraphicsRootShaderResourceView: unsafe extern "C" fn(*mut List, u32, u64),
    SetComputeRootUnorderedAccessView: unsafe extern "C" fn(*mut List, u32, u64),
    SetGraphicsRootUnorderedAccessView: unsafe extern "C" fn(*mut List, u32, u64),
    IASetIndexBuffer: unsafe extern "C" fn(*mut List, usize),
    IASetVertexBuffers: unsafe extern "C" fn(*mut List, u32, u32, usize),
    SOSetTargets: unsafe extern "C" fn(*mut List, u32, u32, usize),
    OMSetRenderTargets: unsafe extern "C" fn(*mut List, u32, usize, i32, usize),
    ClearDepthStencilView: unsafe extern "C" fn(*mut List, CpuHandle, i32, f32, u8, u32, usize),
    ClearRenderTargetView: unsafe extern "C" fn(*mut List, CpuHandle, usize, u32, usize),
    ClearUnorderedAccessViewUint:
        unsafe extern "C" fn(*mut List, GpuHandle, CpuHandle, usize, usize, u32, usize),
    ClearUnorderedAccessViewFloat:
        unsafe extern "C" fn(*mut List, GpuHandle, CpuHandle, usize, usize, u32, usize),
    DiscardResource: unsafe extern "C" fn(*mut List, usize, usize),
    BeginQuery: unsafe extern "C" fn(*mut List, usize, i32, u32),
    EndQuery: unsafe extern "C" fn(*mut List, usize, i32, u32),
    ResolveQueryData: unsafe extern "C" fn(*mut List, usize, i32, u32, u32, usize, u64),
    SetPredication: unsafe extern "C" fn(*mut List, usize, u64, i32),
    SetMarker: unsafe extern "C" fn(*mut List, u32, usize, u32),
    BeginEvent: unsafe extern "C" fn(*mut List, u32, usize, u32),
    EndEvent: unsafe extern "C" fn(*mut List),
    ExecuteIndirect: unsafe extern "C" fn(*mut List, usize, u32, usize, u64, usize, u64),
}
static LIST_VTABLE: ListVtable = ListVtable {
    QueryInterface: list_query,
    AddRef: list_add,
    Release: list_release,
    GetPrivateData: list_get,
    SetPrivateData: list_set,
    SetPrivateDataInterface: list_interface,
    SetName: list_name,
    GetDevice: list_device,
    GetType: kind,
    Close: close,
    Reset: reset,
    ClearState,
    DrawInstanced,
    DrawIndexedInstanced,
    Dispatch,
    CopyBufferRegion: copy_buffer,
    CopyTextureRegion,
    CopyResource: copy_resource,
    CopyTiles,
    ResolveSubresource,
    IASetPrimitiveTopology,
    RSSetViewports,
    RSSetScissorRects,
    OMSetBlendFactor,
    OMSetStencilRef,
    SetPipelineState,
    ResourceBarrier: barriers,
    ExecuteBundle,
    SetDescriptorHeaps,
    SetComputeRootSignature,
    SetGraphicsRootSignature,
    SetComputeRootDescriptorTable,
    SetGraphicsRootDescriptorTable,
    SetComputeRoot32BitConstant,
    SetGraphicsRoot32BitConstant,
    SetComputeRoot32BitConstants,
    SetGraphicsRoot32BitConstants,
    SetComputeRootConstantBufferView,
    SetGraphicsRootConstantBufferView,
    SetComputeRootShaderResourceView,
    SetGraphicsRootShaderResourceView,
    SetComputeRootUnorderedAccessView,
    SetGraphicsRootUnorderedAccessView,
    IASetIndexBuffer,
    IASetVertexBuffers,
    SOSetTargets,
    OMSetRenderTargets,
    ClearDepthStencilView,
    ClearRenderTargetView,
    ClearUnorderedAccessViewUint,
    ClearUnorderedAccessViewFloat,
    DiscardResource,
    BeginQuery,
    EndQuery,
    ResolveQueryData,
    SetPredication,
    SetMarker,
    BeginEvent,
    EndEvent,
    ExecuteIndirect,
};

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
