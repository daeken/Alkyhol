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
| Restricted device/resource constructor | Explicit non-feature-level Metal-backed subset; real shared UPLOAD/READBACK buffers, Map/Unmap/descriptors/parent ownership/GPU addresses; not used by D3D12CreateDevice |
| Restricted queues/fences | Real DIRECT/COMPUTE/COPY Metal queues, CPU fence signals/rewinds, event/null-event waits and ordered native completion callbacks. Finite native WARP comparison passes. No COM command lists, GPU queue waits or timestamps |
| Original queue-management subsystem | Unchanged CommandListManager and CommandAllocatorPool objects execute three-queue creation, high-bit fence routing, increments/waits, idle and shutdown on physical Metal through a separate caller. No allocator creation or command-list execution |
| D3D12 creation | Explicit E_NOTIMPL and cleared interface outputs; no device, resource or command capability success |
| D3D12 root serialization/debug/experiments | Explicit failures; no serialized blob or debug interface |
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
interfaces now compose that implementation. The next runtime steps are command
allocators/lists, actual COM workload submission, descriptor/root-signature and
pipeline support. Neither restricted nor metadata constructors substitute for
production device creation or original application initialization.
