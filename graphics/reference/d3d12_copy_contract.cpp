// Original Windows SDK reference probe. Uses actual D3D12 COM interfaces and
// explicitly selected WARP; it does not link Alkyhol or use its validator.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <d3d12.h>
#include <dxgi1_4.h>
#include <d3dcompiler.h>
#include <wrl/client.h>
#include <cstdio>
#include <cstdint>
#include <climits>
#include <cstring>
#include <stdexcept>
#include <string>
#include <vector>
using Microsoft::WRL::ComPtr;
constexpr UINT width = 65, height = 4;
constexpr DXGI_FORMAT format = DXGI_FORMAT_R8G8B8A8_UNORM;

static void check(HRESULT result, const char* operation) {
    if (FAILED(result)) throw std::runtime_error(std::string(operation) + " HRESULT=" + std::to_string(static_cast<unsigned long>(result)));
}
#define HR(operation) check((operation), #operation)
struct Event {
    HANDLE handle = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    Event() { if (!handle) throw std::runtime_error("CreateEventW failed"); }
    ~Event() { if (handle) CloseHandle(handle); }
};
static D3D12_HEAP_PROPERTIES heap(D3D12_HEAP_TYPE type) {
    D3D12_HEAP_PROPERTIES value = {};
    value.Type = type; value.CreationNodeMask = 1; value.VisibleNodeMask = 1;
    return value;
}
static void transition(ID3D12GraphicsCommandList* list, ID3D12Resource* resource,
                       D3D12_RESOURCE_STATES before, D3D12_RESOURCE_STATES after) {
    D3D12_RESOURCE_BARRIER barrier = {};
    barrier.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    barrier.Transition.pResource = resource;
    barrier.Transition.Subresource = D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;
    barrier.Transition.StateBefore = before; barrier.Transition.StateAfter = after;
    list->ResourceBarrier(1, &barrier);
}
static std::string readback(ID3D12Resource* resource, const D3D12_PLACED_SUBRESOURCE_FOOTPRINT& layout, UINT64 size) {
    void* mapped = nullptr;
    const D3D12_RANGE read_range = {0, static_cast<SIZE_T>(size)};
    HR(resource->Map(0, &read_range, &mapped));
    const auto bytes = static_cast<const unsigned char*>(mapped);
    const char hex[] = "0123456789abcdef";
    std::string result;
    result.reserve(width * height * 8);
    for (UINT y = 0; y < height; ++y) {
        for (UINT x = 0; x < width; ++x) {
            const auto pixel = bytes + layout.Offset + static_cast<SIZE_T>(y) * layout.Footprint.RowPitch + x * 4;
            const unsigned char expected[4] = {static_cast<unsigned char>((x & 1) * 255), static_cast<unsigned char>((y & 1) * 255), static_cast<unsigned char>((((x >> 1) + y) & 1) * 255), 255};
            for (UINT channel = 0; channel < 4; ++channel) {
                if (pixel[channel] != expected[channel]) throw std::runtime_error("unexpected checkerboard pixel");
                result.push_back(hex[pixel[channel] >> 4]); result.push_back(hex[pixel[channel] & 15]);
            }
        }
    }
    const D3D12_RANGE no_writes = {0, 0}; resource->Unmap(0, &no_writes);
    return result;
}

int main() {
    try {
        ComPtr<IDXGIFactory4> factory; HR(CreateDXGIFactory1(IID_PPV_ARGS(&factory)));
        ComPtr<IDXGIAdapter1> adapter; HR(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter)));
        DXGI_ADAPTER_DESC1 description = {}; HR(adapter->GetDesc1(&description));
        if (!(description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE)) throw std::runtime_error("WARP did not identify as software");
        std::fprintf(stderr, "adapter: D3D12 WARP vendor=%u device=%u flags=%u\n", description.VendorId, description.DeviceId, description.Flags);
        ComPtr<ID3D12Device> device; HR(D3D12CreateDevice(adapter.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device)));
        D3D12_COMMAND_QUEUE_DESC queue_desc = {}; queue_desc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
        ComPtr<ID3D12CommandQueue> queue; HR(device->CreateCommandQueue(&queue_desc, IID_PPV_ARGS(&queue)));
        ComPtr<ID3D12CommandAllocator> allocator; HR(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&allocator)));

        D3D12_ROOT_SIGNATURE_DESC root_desc = {};
        root_desc.Flags = D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT;
        ComPtr<ID3DBlob> serialized, errors;
        HR(D3D12SerializeRootSignature(&root_desc, D3D_ROOT_SIGNATURE_VERSION_1, &serialized, &errors));
        ComPtr<ID3D12RootSignature> root;
        HR(device->CreateRootSignature(0, serialized->GetBufferPointer(), serialized->GetBufferSize(), IID_PPV_ARGS(&root)));
        const char* shader = R"(
float4 VSMain(uint id : SV_VertexID) : SV_Position {
    float2 positions[3] = { float2(-1,-1), float2(-1,3), float2(3,-1) };
    return float4(positions[id], 0, 1);
}
float4 PSMain(float4 position : SV_Position) : SV_Target {
    uint x = (uint)position.x, y = (uint)position.y;
    return float4(x & 1, y & 1, ((x >> 1) + y) & 1, 1);
})";
        ComPtr<ID3DBlob> vertex, fragment;
        HR(D3DCompile(shader, std::strlen(shader), "checker", nullptr, nullptr, "VSMain", "vs_5_0", D3DCOMPILE_ENABLE_STRICTNESS, 0, &vertex, &errors));
        HR(D3DCompile(shader, std::strlen(shader), "checker", nullptr, nullptr, "PSMain", "ps_5_0", D3DCOMPILE_ENABLE_STRICTNESS, 0, &fragment, &errors));
        D3D12_GRAPHICS_PIPELINE_STATE_DESC pipeline_desc = {};
        pipeline_desc.pRootSignature = root.Get();
        pipeline_desc.VS = {vertex->GetBufferPointer(), vertex->GetBufferSize()};
        pipeline_desc.PS = {fragment->GetBufferPointer(), fragment->GetBufferSize()};
        pipeline_desc.BlendState.RenderTarget[0] = {FALSE, FALSE, D3D12_BLEND_ONE, D3D12_BLEND_ZERO, D3D12_BLEND_OP_ADD,
            D3D12_BLEND_ONE, D3D12_BLEND_ZERO, D3D12_BLEND_OP_ADD, D3D12_LOGIC_OP_NOOP, D3D12_COLOR_WRITE_ENABLE_ALL};
        pipeline_desc.SampleMask = UINT_MAX;
        pipeline_desc.RasterizerState.FillMode = D3D12_FILL_MODE_SOLID;
        pipeline_desc.RasterizerState.CullMode = D3D12_CULL_MODE_NONE;
        pipeline_desc.RasterizerState.DepthClipEnable = TRUE;
        pipeline_desc.DepthStencilState.DepthEnable = FALSE;
        pipeline_desc.DepthStencilState.DepthWriteMask = D3D12_DEPTH_WRITE_MASK_ZERO;
        pipeline_desc.DepthStencilState.DepthFunc = D3D12_COMPARISON_FUNC_ALWAYS;
        pipeline_desc.DepthStencilState.StencilReadMask = D3D12_DEFAULT_STENCIL_READ_MASK;
        pipeline_desc.DepthStencilState.StencilWriteMask = D3D12_DEFAULT_STENCIL_WRITE_MASK;
        pipeline_desc.DepthStencilState.FrontFace = {D3D12_STENCIL_OP_KEEP, D3D12_STENCIL_OP_KEEP, D3D12_STENCIL_OP_KEEP, D3D12_COMPARISON_FUNC_ALWAYS};
        pipeline_desc.DepthStencilState.BackFace = pipeline_desc.DepthStencilState.FrontFace;
        pipeline_desc.PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE;
        pipeline_desc.NumRenderTargets = 1; pipeline_desc.RTVFormats[0] = format;
        pipeline_desc.SampleDesc = {1, 0};
        ComPtr<ID3D12PipelineState> pipeline; HR(device->CreateGraphicsPipelineState(&pipeline_desc, IID_PPV_ARGS(&pipeline)));

        D3D12_RESOURCE_DESC texture_desc = {};
        texture_desc.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D;
        texture_desc.Width = width; texture_desc.Height = height;
        texture_desc.DepthOrArraySize = 1; texture_desc.MipLevels = 1;
        texture_desc.Format = format; texture_desc.SampleDesc = {1, 0};
        texture_desc.Flags = D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET;
        const auto default_heap = heap(D3D12_HEAP_TYPE_DEFAULT);
        ComPtr<ID3D12Resource> source, destination;
        HR(device->CreateCommittedResource(&default_heap, D3D12_HEAP_FLAG_NONE, &texture_desc, D3D12_RESOURCE_STATE_RENDER_TARGET, nullptr, IID_PPV_ARGS(&source)));
        HR(device->CreateCommittedResource(&default_heap, D3D12_HEAP_FLAG_NONE, &texture_desc, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&destination)));
        D3D12_DESCRIPTOR_HEAP_DESC rtv_desc = {}; rtv_desc.Type = D3D12_DESCRIPTOR_HEAP_TYPE_RTV; rtv_desc.NumDescriptors = 1;
        ComPtr<ID3D12DescriptorHeap> rtv_heap; HR(device->CreateDescriptorHeap(&rtv_desc, IID_PPV_ARGS(&rtv_heap)));
        const auto rtv = rtv_heap->GetCPUDescriptorHandleForHeapStart(); device->CreateRenderTargetView(source.Get(), nullptr, rtv);

        D3D12_PLACED_SUBRESOURCE_FOOTPRINT layout = {}; UINT64 bytes = 0, row_bytes = 0; UINT rows = 0;
        device->GetCopyableFootprints(&texture_desc, 0, 1, 0, &layout, &rows, &row_bytes, &bytes);
        if (rows != height || row_bytes != width * 4 || bytes == UINT64_MAX || layout.Footprint.RowPitch < row_bytes) throw std::runtime_error("invalid copy footprint");
        D3D12_RESOURCE_DESC buffer_desc = {}; buffer_desc.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER;
        buffer_desc.Width = bytes; buffer_desc.Height = 1; buffer_desc.DepthOrArraySize = 1; buffer_desc.MipLevels = 1;
        buffer_desc.SampleDesc = {1, 0}; buffer_desc.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR;
        const auto readback_heap = heap(D3D12_HEAP_TYPE_READBACK);
        ComPtr<ID3D12Resource> source_buffer, destination_buffer;
        HR(device->CreateCommittedResource(&readback_heap, D3D12_HEAP_FLAG_NONE, &buffer_desc, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&source_buffer)));
        HR(device->CreateCommittedResource(&readback_heap, D3D12_HEAP_FLAG_NONE, &buffer_desc, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&destination_buffer)));

        ComPtr<ID3D12GraphicsCommandList> list;
        HR(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, allocator.Get(), pipeline.Get(), IID_PPV_ARGS(&list)));
        list->SetGraphicsRootSignature(root.Get());
        const D3D12_VIEWPORT viewport = {0, 0, static_cast<float>(width), static_cast<float>(height), 0, 1};
        const D3D12_RECT scissor = {0, 0, static_cast<LONG>(width), static_cast<LONG>(height)};
        list->RSSetViewports(1, &viewport); list->RSSetScissorRects(1, &scissor);
        list->OMSetRenderTargets(1, &rtv, FALSE, nullptr);
        const float green[4] = {0, 1, 0, 1}; list->ClearRenderTargetView(rtv, green, 0, nullptr);
        list->IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST); list->DrawInstanced(3, 1, 0, 0);
        transition(list.Get(), source.Get(), D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_SOURCE);
        list->CopyResource(destination.Get(), source.Get());
        transition(list.Get(), destination.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE);
        for (unsigned i = 0; i < 2; ++i) {
            D3D12_TEXTURE_COPY_LOCATION from = {}, to = {};
            from.pResource = i == 0 ? source.Get() : destination.Get(); from.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
            to.pResource = i == 0 ? source_buffer.Get() : destination_buffer.Get(); to.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT; to.PlacedFootprint = layout;
            list->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
        }
        HR(list->Close()); ID3D12CommandList* lists[] = {list.Get()}; queue->ExecuteCommandLists(1, lists);
        ComPtr<ID3D12Fence> fence; HR(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence)));
        Event done; HR(queue->Signal(fence.Get(), 1)); HR(fence->SetEventOnCompletion(1, done.handle));
        if (WaitForSingleObject(done.handle, 10000) != WAIT_OBJECT_0 || fence->GetCompletedValue() != 1) throw std::runtime_error("GPU fence failed or timed out");
        const auto original = readback(source_buffer.Get(), layout, bytes);
        const auto copied = readback(destination_buffer.Get(), layout, bytes);
        if (original != copied) throw std::runtime_error("CopyResource source/destination differ");
        std::printf("{\"scenario\":\"rgba8-checker-copy-v1\",\"width\":%u,\"height\":%u,\"source_rgba8\":\"%s\",\"destination_rgba8\":\"%s\"}\n", width, height, original.c_str(), copied.c_str());
        return 0;
    } catch (const std::exception& error) { std::fprintf(stderr, "%s\n", error.what()); return 1; }
}
