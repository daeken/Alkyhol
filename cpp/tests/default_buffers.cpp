// Copyright (c) Alkyhol contributors. MIT licensed.
// Identical native WARP / restricted Metal DEFAULT-buffer transition workload.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <vector>
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
static ComPtr<ID3D12Resource> buffer(ID3D12Device* device,D3D12_HEAP_TYPE type,D3D12_RESOURCE_STATES state){
    ComPtr<ID3D12Resource> result;D3D12_HEAP_PROPERTIES heap={type,D3D12_CPU_PAGE_PROPERTY_UNKNOWN,D3D12_MEMORY_POOL_UNKNOWN,1,1};
    D3D12_RESOURCE_DESC desc={D3D12_RESOURCE_DIMENSION_BUFFER,0,1024,1,1,1,DXGI_FORMAT_UNKNOWN,{1,0},D3D12_TEXTURE_LAYOUT_ROW_MAJOR,D3D12_RESOURCE_FLAG_NONE};
    assert(device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,state,nullptr,IID_PPV_ARGS(&result))==S_OK);return result;
}
static void transition(ID3D12GraphicsCommandList* list,ID3D12Resource* resource,D3D12_RESOURCE_STATES before,D3D12_RESOURCE_STATES after,UINT subresource=D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES){
    D3D12_RESOURCE_BARRIER b{};b.Type=D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;b.Flags=D3D12_RESOURCE_BARRIER_FLAG_NONE;b.Transition.pResource=resource;b.Transition.Subresource=subresource;b.Transition.StateBefore=before;b.Transition.StateAfter=after;list->ResourceBarrier(1,&b);
}
static unsigned char pattern(unsigned i,bool second){return static_cast<unsigned char>(second?(i*7+31):(i*13+3));}
static void fill(ID3D12Resource* resource,bool second){void* mapped=nullptr;assert(resource->Map(0,nullptr,&mapped)==S_OK);for(unsigned i=0;i<1024;++i)static_cast<unsigned char*>(mapped)[i]=pattern(i,second);resource->Unmap(0,nullptr);}
static void observe(ID3D12Resource* resource,unsigned index){
    void* mapped=nullptr;assert(resource->Map(0,nullptr,&mapped)==S_OK);const auto* data=static_cast<const unsigned char*>(mapped);std::putchar('"');
    for(unsigned i=0;i<1024;++i){unsigned char expected=0;if(index==0)expected=(i>=16&&i<272)?pattern(i-12,false):0xa5;else if(index==1)expected=pattern(i,false);else if(index==2)expected=(i>=128&&i<384)?pattern(i+384,true):pattern(i,false);else expected=pattern(i,true);assert(data[i]==expected);std::printf("%02x",unsigned(data[i]));}std::putchar('"');resource->Unmap(0,nullptr);
}
int main(int argc,char** argv){
    (void)argc;(void)argv;static_assert(sizeof(D3D12_RESOURCE_BARRIER)==32,"transition source ABI");
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    auto upload=buffer(device.Get(),D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ);auto a=buffer(device.Get(),D3D12_HEAP_TYPE_DEFAULT,D3D12_RESOURCE_STATE_COPY_DEST);auto b=buffer(device.Get(),D3D12_HEAP_TYPE_DEFAULT,D3D12_RESOURCE_STATE_COMMON);
    void* forbidden=nullptr;const HRESULT map_hr=a->Map(0,nullptr,&forbidden);assert(FAILED(map_hr));
#ifndef _WIN32
    if(argc==2){if(!std::strcmp(argv[1],"trap-default-unmap"))a->Unmap(0,nullptr);return 99;}
#endif
    D3D12_HEAP_PROPERTIES heap{};D3D12_HEAP_FLAGS heap_flags{};assert(a->GetHeapProperties(&heap,&heap_flags)==S_OK);const auto desc=a->GetDesc();assert(a->GetGPUVirtualAddress()!=0);
    ComPtr<ID3D12Resource> readback[4];for(auto& r:readback){r=buffer(device.Get(),D3D12_HEAP_TYPE_READBACK,D3D12_RESOURCE_STATE_COPY_DEST);void* mapped=nullptr;assert(r->Map(0,nullptr,&mapped)==S_OK);std::memset(mapped,0xa5,1024);r->Unmap(0,nullptr);}
    fill(upload.Get(),false);
    ComPtr<ID3D12CommandQueue> direct,copy;for(auto type:{D3D12_COMMAND_LIST_TYPE_DIRECT,D3D12_COMMAND_LIST_TYPE_COPY}){D3D12_COMMAND_QUEUE_DESC q={type,0,D3D12_COMMAND_QUEUE_FLAG_NONE,1};ComPtr<ID3D12CommandQueue> result;assert(device->CreateCommandQueue(&q,IID_PPV_ARGS(&result))==S_OK);if(type==D3D12_COMMAND_LIST_TYPE_DIRECT)direct=result;else copy=result;}
    ComPtr<ID3D12CommandAllocator> direct_allocator,copy_allocator,tail_allocator;assert(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT,IID_PPV_ARGS(&direct_allocator))==S_OK);assert(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_COPY,IID_PPV_ARGS(&copy_allocator))==S_OK);assert(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT,IID_PPV_ARGS(&tail_allocator))==S_OK);
    ComPtr<ID3D12GraphicsCommandList> list,copier,tail;assert(device->CreateCommandList(1,D3D12_COMMAND_LIST_TYPE_DIRECT,direct_allocator.Get(),nullptr,IID_PPV_ARGS(&list))==S_OK);assert(device->CreateCommandList(1,D3D12_COMMAND_LIST_TYPE_COPY,copy_allocator.Get(),nullptr,IID_PPV_ARGS(&copier))==S_OK);assert(device->CreateCommandList(1,D3D12_COMMAND_LIST_TYPE_DIRECT,tail_allocator.Get(),nullptr,IID_PPV_ARGS(&tail))==S_OK);
    ComPtr<ID3D12Fence> fence;assert(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&fence))==S_OK);HANDLE event=CreateEventW(nullptr,FALSE,FALSE,nullptr);assert(event);std::vector<unsigned long long> values;
    auto finish=[&](ID3D12CommandQueue* queue,UINT64 value){assert(fence->SetEventOnCompletion(value,event)==S_OK&&queue->Signal(fence.Get(),value)==S_OK);assert(WaitForSingleObject(event,5000)==WAIT_OBJECT_0);values.push_back(fence->GetCompletedValue());assert(values.back()==value);};
    list->CopyResource(a.Get(),upload.Get());transition(list.Get(),a.Get(),D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_STATE_COPY_SOURCE);list->CopyResource(b.Get(),a.Get());transition(list.Get(),b.Get(),D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_STATE_COPY_SOURCE,0);list->CopyBufferRegion(readback[0].Get(),16,b.Get(),4,256);assert(list->Close()==S_OK);ID3D12CommandList* first[]={list.Get()};direct->ExecuteCommandLists(1,first);finish(direct.Get(),1);
    // Both DEFAULT buffers decayed to COMMON at the prior execution boundary.
    copier->CopyResource(readback[1].Get(),a.Get());assert(copier->Close()==S_OK);ID3D12CommandList* handoff[]={copier.Get()};copy->ExecuteCommandLists(1,handoff);finish(copy.Get(),2);
    fill(upload.Get(),true);assert(direct_allocator->Reset()==S_OK&&list->Reset(direct_allocator.Get(),nullptr)==S_OK);
    list->CopyBufferRegion(a.Get(),128,upload.Get(),512,256);assert(list->Close()==S_OK);
    // Decay is between ExecuteCommandLists calls, never between these two lists.
    transition(tail.Get(),a.Get(),D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_STATE_COPY_SOURCE);tail->CopyResource(readback[2].Get(),a.Get());
    // A promotable old BeforeState is valid even after the previous buffer decay.
    transition(tail.Get(),b.Get(),D3D12_RESOURCE_STATE_COPY_SOURCE,D3D12_RESOURCE_STATE_COPY_DEST);tail->CopyResource(b.Get(),upload.Get());transition(tail.Get(),b.Get(),D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_STATE_COPY_SOURCE);tail->CopyResource(readback[3].Get(),b.Get());assert(tail->Close()==S_OK);ID3D12CommandList* batch[]={list.Get(),tail.Get()};direct->ExecuteCommandLists(2,batch);finish(direct.Get(),3);assert(CloseHandle(event));
    std::printf("{\"scenario\":\"default-buffer-transitions-v1\",\"map_hr\":%u,\"heap\":[%u,%u,%u,%u,%u],\"heap_flags\":%u,\"desc\":[%u,%llu,%llu,%u,%u,%u,%u,%u,%u,%u,%u],\"fences\":[",unsigned(map_hr),unsigned(heap.Type),unsigned(heap.CPUPageProperty),unsigned(heap.MemoryPoolPreference),unsigned(heap.CreationNodeMask),unsigned(heap.VisibleNodeMask),unsigned(heap_flags),unsigned(desc.Dimension),static_cast<unsigned long long>(desc.Alignment),static_cast<unsigned long long>(desc.Width),unsigned(desc.Height),unsigned(desc.DepthOrArraySize),unsigned(desc.MipLevels),unsigned(desc.Format),unsigned(desc.SampleDesc.Count),unsigned(desc.SampleDesc.Quality),unsigned(desc.Layout),unsigned(desc.Flags));
    for(size_t i=0;i<values.size();++i)std::printf("%s%llu",i?",":"",values[i]);std::printf("],\"buffers\":[");for(unsigned i=0;i<4;++i){if(i)std::putchar(',');observe(readback[i].Get(),i);}std::puts("]}");
}
