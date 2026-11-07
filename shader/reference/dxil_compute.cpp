// Copyright (c) Alkyhol contributors. MIT licensed.
// Independent Windows SDK/D3D12 execution. Consumes the exact same DXIL bytes as
// the Metal translator, using WARP explicitly; no Alkyhol library is linked.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <d3d12.h>
#include <dxgi1_4.h>
#include <wrl/client.h>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <stdexcept>
#include <string>
#include <vector>
using Microsoft::WRL::ComPtr;
static void check(HRESULT value, const char* operation) {
    if (FAILED(value)) throw std::runtime_error(std::string(operation) + " HRESULT=" + std::to_string(static_cast<unsigned long>(value)));
}
#define HR(operation) check((operation), #operation)
struct Event {
    HANDLE handle = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    Event() { if (!handle) throw std::runtime_error("CreateEventW failed"); }
    ~Event() { if (handle) CloseHandle(handle); }
};
static ComPtr<ID3D12Resource> buffer(ID3D12Device* device, UINT64 bytes,
    D3D12_HEAP_TYPE type, D3D12_RESOURCE_STATES state, D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE) {
    D3D12_HEAP_PROPERTIES heap = {}; heap.Type = type; heap.CreationNodeMask = 1; heap.VisibleNodeMask = 1;
    D3D12_RESOURCE_DESC desc = {}; desc.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER;
    desc.Width = bytes; desc.Height = 1; desc.DepthOrArraySize = 1; desc.MipLevels = 1;
    desc.SampleDesc = {1, 0}; desc.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR; desc.Flags = flags;
    ComPtr<ID3D12Resource> result;
    HR(device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &desc, state, nullptr, IID_PPV_ARGS(&result)));
    return result;
}
static void upload(ID3D12Resource* resource, const void* source, SIZE_T bytes) {
    void* mapped = nullptr; const D3D12_RANGE no_reads = {0, 0};
    HR(resource->Map(0, &no_reads, &mapped)); std::memcpy(mapped, source, bytes);
    const D3D12_RANGE writes = {0, bytes}; resource->Unmap(0, &writes);
}
static void transition(ID3D12GraphicsCommandList* list, ID3D12Resource* resource,
    D3D12_RESOURCE_STATES before, D3D12_RESOURCE_STATES after) {
    D3D12_RESOURCE_BARRIER barrier = {}; barrier.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    barrier.Transition.pResource = resource; barrier.Transition.Subresource = D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;
    barrier.Transition.StateBefore = before; barrier.Transition.StateAfter = after; list->ResourceBarrier(1, &barrier);
}
int main(int argc, char** argv) {
    try {
        if (argc != 2) throw std::runtime_error("usage: dxil_compute SHADER.dxil");
        std::ifstream input(argv[1], std::ios::binary | std::ios::ate);
        const auto size = input.tellg();
        if (!input || size < 32 || size > 16 * 1024 * 1024) throw std::runtime_error("invalid DXIL file size");
        std::vector<char> dxil(static_cast<size_t>(size)); input.seekg(0);
        if (!input.read(dxil.data(), static_cast<std::streamsize>(size))) throw std::runtime_error("DXIL read failed");
        ComPtr<IDXGIFactory4> factory; HR(CreateDXGIFactory1(IID_PPV_ARGS(&factory)));
        ComPtr<IDXGIAdapter1> adapter; HR(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter)));
        DXGI_ADAPTER_DESC1 description = {}; HR(adapter->GetDesc1(&description));
        if (!(description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE)) throw std::runtime_error("WARP did not identify as software");
        std::fprintf(stderr, "adapter: native D3D12 WARP vendor=%u device=%u flags=%u\n", description.VendorId, description.DeviceId, description.Flags);
        ComPtr<ID3D12Device> device; HR(D3D12CreateDevice(adapter.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device)));
        D3D12_FEATURE_DATA_SHADER_MODEL model = {D3D_SHADER_MODEL_6_0};
        HR(device->CheckFeatureSupport(D3D12_FEATURE_SHADER_MODEL, &model, sizeof(model)));
        if (model.HighestShaderModel < D3D_SHADER_MODEL_6_0) throw std::runtime_error("WARP lacks shader model 6.0");
        D3D12_ROOT_PARAMETER params[4] = {};
        const D3D12_ROOT_PARAMETER_TYPE kinds[4] = {D3D12_ROOT_PARAMETER_TYPE_CBV,
            D3D12_ROOT_PARAMETER_TYPE_SRV, D3D12_ROOT_PARAMETER_TYPE_SRV, D3D12_ROOT_PARAMETER_TYPE_UAV};
        for (unsigned i = 0; i < 4; ++i) {
            params[i].ParameterType = kinds[i]; params[i].ShaderVisibility = D3D12_SHADER_VISIBILITY_ALL;
            params[i].Descriptor.ShaderRegister = 0; params[i].Descriptor.RegisterSpace = i == 2 ? 2 : 0;
        }
        D3D12_ROOT_SIGNATURE_DESC signature = {}; signature.NumParameters = 4; signature.pParameters = params;
        ComPtr<ID3DBlob> serialized, errors;
        HR(D3D12SerializeRootSignature(&signature, D3D_ROOT_SIGNATURE_VERSION_1, &serialized, &errors));
        ComPtr<ID3D12RootSignature> root;
        HR(device->CreateRootSignature(0, serialized->GetBufferPointer(), serialized->GetBufferSize(), IID_PPV_ARGS(&root)));
        D3D12_COMPUTE_PIPELINE_STATE_DESC pipeline_desc = {}; pipeline_desc.pRootSignature = root.Get();
        pipeline_desc.CS = {dxil.data(), dxil.size()};
        ComPtr<ID3D12PipelineState> pipeline; HR(device->CreateComputePipelineState(&pipeline_desc, IID_PPV_ARGS(&pipeline)));
        constexpr UINT count = 19; constexpr SIZE_T output_bytes = 24 * 4 * sizeof(std::uint32_t);
        std::array<float, 16> constants = {0, 0, 2.f, .5f, 1, 1, 1, 1, 2, 3, 4, 5, -1, 2, -3, 4};
        const std::uint32_t integers[2] = {count, 0x12345678}; std::memcpy(constants.data(), integers, sizeof(integers));
        std::array<std::uint32_t, count> a = {}, b = {};
        for (UINT i = 0; i < count; ++i) {a[i] = 0xfffffff0u + i; b[i] = 3 * i + 7;}
        std::array<std::uint32_t, 96> sentinel; sentinel.fill(0xdeadbeef);
        auto cb = buffer(device.Get(), 256, D3D12_HEAP_TYPE_UPLOAD, D3D12_RESOURCE_STATE_GENERIC_READ);
        auto aa = buffer(device.Get(), sizeof(a), D3D12_HEAP_TYPE_UPLOAD, D3D12_RESOURCE_STATE_GENERIC_READ);
        auto bb = buffer(device.Get(), sizeof(b), D3D12_HEAP_TYPE_UPLOAD, D3D12_RESOURCE_STATE_GENERIC_READ);
        auto initial = buffer(device.Get(), output_bytes, D3D12_HEAP_TYPE_UPLOAD, D3D12_RESOURCE_STATE_GENERIC_READ);
        auto output = buffer(device.Get(), output_bytes, D3D12_HEAP_TYPE_DEFAULT, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS);
        auto readback = buffer(device.Get(), output_bytes, D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST);
        upload(cb.Get(), constants.data(), sizeof(constants)); upload(aa.Get(), a.data(), sizeof(a));
        upload(bb.Get(), b.data(), sizeof(b)); upload(initial.Get(), sentinel.data(), sizeof(sentinel));
        D3D12_COMMAND_QUEUE_DESC queue_desc = {}; queue_desc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
        ComPtr<ID3D12CommandQueue> queue; HR(device->CreateCommandQueue(&queue_desc, IID_PPV_ARGS(&queue)));
        ComPtr<ID3D12CommandAllocator> allocator; HR(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&allocator)));
        ComPtr<ID3D12GraphicsCommandList> list;
        HR(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, allocator.Get(), pipeline.Get(), IID_PPV_ARGS(&list)));
        list->CopyResource(output.Get(), initial.Get());
        transition(list.Get(), output.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
        list->SetComputeRootSignature(root.Get()); list->SetComputeRootConstantBufferView(0, cb->GetGPUVirtualAddress());
        list->SetComputeRootShaderResourceView(1, aa->GetGPUVirtualAddress());
        list->SetComputeRootShaderResourceView(2, bb->GetGPUVirtualAddress());
        list->SetComputeRootUnorderedAccessView(3, output->GetGPUVirtualAddress()); list->Dispatch(3, 1, 1);
        transition(list.Get(), output.Get(), D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_STATE_COPY_SOURCE);
        list->CopyResource(readback.Get(), output.Get()); HR(list->Close());
        ID3D12CommandList* lists[] = {list.Get()}; queue->ExecuteCommandLists(1, lists);
        ComPtr<ID3D12Fence> fence; HR(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence)));
        Event done; HR(queue->Signal(fence.Get(), 1)); HR(fence->SetEventOnCompletion(1, done.handle));
        if (WaitForSingleObject(done.handle, 10000) != WAIT_OBJECT_0 || fence->GetCompletedValue() != 1) throw std::runtime_error("GPU fence failed or timed out");
        void* mapped = nullptr; const D3D12_RANGE reads = {0, output_bytes}; HR(readback->Map(0, &reads, &mapped));
        std::array<std::uint32_t, 96> words; std::memcpy(words.data(), mapped, sizeof(words));
        const D3D12_RANGE no_writes = {0, 0}; readback->Unmap(0, &no_writes);
        std::printf("{\"scenario\":\"dxil-compute-v1\",\"count\":19,\"output_words\":[");
        for (size_t i = 0; i < words.size(); ++i) std::printf("%s%u", i ? "," : "", words[i]);
        std::printf("]}\n"); return 0;
    } catch (const std::exception& error) { std::fprintf(stderr, "%s\n", error.what()); return 1; }
}
