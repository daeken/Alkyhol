# Original C++ runtime boundary

The complete original MiniEngine Sponza application compiles and links. Its
startup bridge currently exits 78 before original initialization because the
production D3D12CreateDevice contract is unavailable. A separately named restricted
Metal COM subset is implemented and tested; this is not a functioning original
renderer or a feature-level claim.

| Area | Executed implementation / limit |
|---|---|
| IUnknown / ID3DBlob | Rust vtable, canonical identity, QueryInterface, owned allocation; synchronized C++/WRL probes agree with native Windows. Rust atomic stress passes separately; native uncoordinated stress crashed and is not parity evidence |
| ID3D12Object metadata base | Real Rust CPU vtable/private bytes/retained interfaces/UTF-32 names; 34 frozen native records match locally. Explicit diagnostic constructor only; no device/fence/resource capability |
| RoInitialize / RoUninitialize | Per-thread model/nesting, changed-mode failure; no WinRT activation or marshaling |
| CreateFile2 | UTF-32 host path conversion to existing real kernel files; synchronous regular files, share mask **7 only**, normal attributes, optional sequential hint; security/templates/other flags fail |
| FileStandardInfo | Actual open-file size, host allocated blocks and link count; no delete-on-close mode; unsupported info classes fail |
| Module identity/path | Current executable token and actual executable path, bounded output/truncation; other modules unsupported |
| Text conversion | UTF-32 source ABI, UTF-8 and ASCII subset of CP_ACP; unsupported pages/flags/default substitutions fail; core UTF-16 Win32 ABI unchanged |
| CRT/source headers | Wide host paths/streams/stat, bounded secure fields, aligned allocation, async tasks; not a complete UCRT implementation |
| Wide printf | Windows wide string/character conventions used by original helpers, including `%s`, `%S`, `%ws`; bounded formats, not all MSVC printf extensions |
| DirectInput public tables | 256 keyboard and 11 mouse object descriptors with public GUIDs; metadata only |
| DXGI discovery | Real Metal factory through IDXGIFactory6 and adapter through IDXGIAdapter1; retained devices, parent ownership, descriptors, metadata and LUID lookup; no swap chains/outputs/device capability |
| Restricted device/resource constructor | Explicit non-feature-level Metal-backed subset; real Shared UPLOAD/READBACK and Private DEFAULT buffers, descriptors/parent ownership/actual aligned GPU addresses. DEFAULT is not CPU-mappable. Not used by D3D12CreateDevice |
| Restricted queues/fences | Real DIRECT/COMPUTE/COPY Metal queues, CPU fence signals/rewinds, event/null-event waits and ordered native completion callbacks. Finite native WARP fence comparison passes. GPU queue waits/timestamps remain unavailable |
| Restricted allocators/command lists | Real state/epoch/snapshot ownership and COM submission of buffer copies. The Shared-copy probe's 2048 bytes/eleven state results match native WARP, hosted Metal and physical M4. Drawing, dispatch and textures remain unsupported; bounded buffer barriers and byte lowering are below |
| DEFAULT transitions | DIRECT-list buffer transitions, bounded COMMON/read/COPY promotion/decay and atomic batch state validation. Private-buffer copy chains and CPU-fenced queue handoff match native WARP, hosted Metal and physical M4 on all 4096 bytes/returned fields. Split/alias/UAV barriers and GPU queue waits remain unavailable |
| Byte-granular copies | Aligned blits plus an open native MSL byte-copy utility, with exact ranges, cached pipeline and bounded dispatch chunks. Native WARP, hosted Metal and physical M4 agree on 272 cases/Private overlays, all small readbacks, and large-copy checksum/windows with full local range assertions. No general D3D Dispatch or shader translation claim |
| Descriptor-heap allocation | Real 11-slot COM objects and empty Metal-backed slots for four CPU/two visible types; metadata/parent ownership and own backend stride. Allocation/handle invariants match native WARP, hosted Metal and physical M4; original allocator/heap helpers execute locally. Visibility is flags, not handle bits. Initialized CBVs/copies are below; general binding unavailable |
| CBVs/descriptor copies | Non-null aligned UPLOAD CBVs and transactional simple/range copies with copied-value ownership. Physical M4 GPU diagnostic reads all copied records/guards. All 72 content/guard words match native WARP, hosted Metal and physical M4 (frozen native report); no general root signatures/PSOs/Dispatch |
| Original queue-management subsystem | Unchanged CommandListManager and CommandAllocatorPool objects execute creation, allocator allocation/deferred reuse/reset, three-queue COM copies, high-bit fences/waits, idle and shutdown on physical Metal through a separate caller |
| D3D12 creation | Explicit E_NOTIMPL and cleared interface outputs; no device, resource or command capability success |
| Root serialization/COM layout | Bounded version-1.0 standalone RTS0/DXBC serialization, owned layouts and all eight root-object slots. All 980 serialized bytes and COM assertions match native WARP, hosted Rust/Metal and physical M4; frozen native report. Original RootSignature.cpp Finalize/cache/static-sampler helper now runs. No GPU binding/PSO/Dispatch |
| Compute dispatch | Real DIRECT/COMPUTE COM submission with root buffer descriptors, Private UAVs, state validation/barriers, initial/reset PSOs and readback. All 624 data/guard words match native WARP and physical M4 from identical source/DXIL; native report frozen, hosted comparison pending. Tables/constants/textures/embedded roots and graphics remain unavailable |
| Compute PSO creation | Real nine-slot COM object and native Metal executable from open DXIL lowering; single-buffer resources covered by owned roots. Original ComputePSO finalization/cache helper runs with the established test shader. Physical GPU unit checks 96 words; common construction/COM assertions match native WARP, hosted Metal and physical M4 (frozen native record). Embedded roots, graphics PSOs, tables/constants and cache blobs remain unavailable |
| D3D12 debug/experiments | Explicit failures; no debug interface |
| Window/message/input services | Explicit unavailable diagnostics/failure returns; no simulated window, silent event loop or successful input device |
| Command-line shell services | Explicit unavailable diagnostics/failure outputs |

File allocation sizes and path behavior describe this host, not Windows filesystem
parity. Native 32-bit wchar_t is Microsoft's Linux header adaptation strategy;
it is not Windows binary layout compatibility. Source-level extensions and header
contracts remain separately visible from portable Windows SDK comparisons.

The launcher does not automatically substitute the Rust scene port. Its existing
Metal backend is not yet connected through the original application's COM device,
command lists, descriptors, swap chain and DXR shader tables. Those are the next
runtime milestones. Full original shader library lowering is also still separate
from compiling the original libraries to DXIL.

The inherited metadata unit now has a real CPU Rust implementation and an explicit
ABI-test constructor. [ID3D12Object metadata](reference/README.md) preserves its
independent native contract. Restricted derived device/resource/queue/fence
interfaces now compose that implementation. Restricted command-list GPU copies
also work, as do bounded Private-buffer transitions. The next runtime steps
include remaining descriptor views/binding and pipeline support.
Neither restricted nor metadata constructors substitute for
production device creation or original application initialization.
