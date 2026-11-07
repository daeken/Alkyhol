// Copyright (c) Alkyhol contributors. MIT licensed.
// Finite, identical native WARP / restricted Metal COM buffer-copy workload.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <vector>
#include <memory>
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
static unsigned char pattern(unsigned i){return static_cast<unsigned char>((i*29+7)&255);}
static void hex(ID3D12Resource* resource){
    void* mapped=nullptr;assert(resource->Map(0,nullptr,&mapped)==S_OK);
    const auto* bytes=static_cast<const unsigned char*>(mapped);std::putchar('"');
    for(unsigned i=0;i<1024;++i)std::printf("%02x",unsigned(bytes[i]));std::putchar('"');resource->Unmap(0,nullptr);
}
int main(int argc,char** argv){
    (void)argc;(void)argv;
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;
    assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    ComPtr<IUnknown> identity;assert(device.As(&identity)==S_OK);IUnknown* canonical=identity.Get();identity.Reset();factory.Reset();adapter.Reset();
    D3D12_COMMAND_QUEUE_DESC desc={D3D12_COMMAND_LIST_TYPE_COPY,0,D3D12_COMMAND_QUEUE_FLAG_NONE,1};ComPtr<ID3D12CommandQueue> queue;assert(device->CreateCommandQueue(&desc,IID_PPV_ARGS(&queue))==S_OK);
    ComPtr<ID3D12CommandAllocator> first,second,whole;for(auto* allocator:{std::addressof(first),std::addressof(second),std::addressof(whole)})assert(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_COPY,IID_PPV_ARGS(allocator->GetAddressOf()))==S_OK);
    assert(first->SetName(L"copy allocator")==S_OK);
    ComPtr<ID3D12GraphicsCommandList> list,full;
    assert(device->CreateCommandList(1,D3D12_COMMAND_LIST_TYPE_COPY,first.Get(),nullptr,IID_PPV_ARGS(&list))==S_OK);assert(list->GetType()==D3D12_COMMAND_LIST_TYPE_COPY);assert(list->SetName(L"copy list")==S_OK);
#ifndef _WIN32
    if(argc==2){
        if(!std::strcmp(argv[1],"trap-draw"))list->DrawInstanced(1,1,0,0);
        if(!std::strcmp(argv[1],"trap-dispatch"))list->Dispatch(1,1,1);
        if(!std::strcmp(argv[1],"trap-barrier"))list->ResourceBarrier(0,nullptr);
        if(!std::strcmp(argv[1],"trap-clear"))list->ClearDepthStencilView({0},D3D12_CLEAR_FLAG_DEPTH,0.5f,1,0,nullptr);
        if(!std::strcmp(argv[1],"trap-root"))list->SetComputeRootDescriptorTable(0,{0});
        if(!std::strcmp(argv[1],"trap-indirect"))list->ExecuteIndirect(nullptr,1,nullptr,0,nullptr,0);
        return 99;
    }
#endif
    ComPtr<ID3D12CommandList> base;ComPtr<ID3D12DeviceChild> child;ComPtr<ID3D12Pageable> pageable;
    assert(list.As(&base)==S_OK&&list.As(&child)==S_OK&&list.As(&pageable)==E_NOINTERFACE);assert(first.As(&pageable)==S_OK);
    ComPtr<IUnknown> list_identity,base_identity;assert(list.As(&list_identity)==S_OK&&base.As(&base_identity)==S_OK&&list_identity.Get()==base_identity.Get());
    std::vector<unsigned> results;results.push_back(unsigned(first->Reset()));results.push_back(unsigned(list->Reset(second.Get(),nullptr)));
    results.push_back(unsigned(list->Close()));results.push_back(unsigned(first->Reset()));results.push_back(unsigned(list->Reset(first.Get(),nullptr)));
    assert(results[0]==unsigned(E_FAIL)&&results[1]==unsigned(E_FAIL));for(unsigned i=2;i<results.size();++i)assert(results[i]==unsigned(S_OK));
    ComPtr<ID3D12Resource> upload,partial,complete;
    for(auto* resource:{std::addressof(upload),std::addressof(partial),std::addressof(complete)}){
        const bool is_upload=resource==std::addressof(upload);
        D3D12_HEAP_PROPERTIES heap={is_upload?D3D12_HEAP_TYPE_UPLOAD:D3D12_HEAP_TYPE_READBACK,D3D12_CPU_PAGE_PROPERTY_UNKNOWN,D3D12_MEMORY_POOL_UNKNOWN,1,1};
        D3D12_RESOURCE_DESC buffer={D3D12_RESOURCE_DIMENSION_BUFFER,0,1024,1,1,1,DXGI_FORMAT_UNKNOWN,{1,0},D3D12_TEXTURE_LAYOUT_ROW_MAJOR,D3D12_RESOURCE_FLAG_NONE};
        assert(device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&buffer,is_upload?D3D12_RESOURCE_STATE_GENERIC_READ:D3D12_RESOURCE_STATE_COPY_DEST,nullptr,IID_PPV_ARGS(resource->GetAddressOf()))==S_OK);
        void* mapped=nullptr;assert((*resource)->Map(0,nullptr,&mapped)==S_OK);std::memset(mapped,is_upload?0x11:0xa5,1024);(*resource)->Unmap(0,nullptr);
    }
    assert(device->CreateCommandList(1,D3D12_COMMAND_LIST_TYPE_COPY,whole.Get(),nullptr,IID_PPV_ARGS(&full))==S_OK);
    list->CopyBufferRegion(partial.Get(),32,upload.Get(),4,256);full->CopyResource(complete.Get(),upload.Get());results.push_back(unsigned(list->Close()));results.push_back(unsigned(full->Close()));
    // Recording must neither execute a CPU copy nor capture upload contents.
    void* mapped=nullptr;assert(partial->Map(0,nullptr,&mapped)==S_OK);for(unsigned i=0;i<1024;++i)assert(static_cast<unsigned char*>(mapped)[i]==0xa5);partial->Unmap(0,nullptr);
    assert(upload->Map(0,nullptr,&mapped)==S_OK);for(unsigned i=0;i<1024;++i)static_cast<unsigned char*>(mapped)[i]=pattern(i);upload->Unmap(0,nullptr);
    ID3D12CommandList* first_batch[]={list.Get()};queue->ExecuteCommandLists(1,first_batch);
    // Reset replaces CPU recording while the previous immutable snapshot may run.
    results.push_back(unsigned(list->Reset(second.Get(),nullptr)));list->CopyBufferRegion(partial.Get(),512,upload.Get(),256,128);results.push_back(unsigned(list->Close()));
    ID3D12CommandList* second_batch[]={list.Get(),full.Get()};queue->ExecuteCommandLists(2,second_batch);
    ComPtr<ID3D12Fence> fence;assert(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&fence))==S_OK);HANDLE event=CreateEventW(nullptr,FALSE,FALSE,nullptr);assert(event);
    assert(fence->SetEventOnCompletion(1,event)==S_OK&&queue->Signal(fence.Get(),1)==S_OK);assert(WaitForSingleObject(event,5000)==WAIT_OBJECT_0);assert(CloseHandle(event));
    results.push_back(unsigned(first->Reset()));results.push_back(unsigned(second->Reset()));for(unsigned i=2;i<results.size();++i)assert(results[i]==unsigned(S_OK));
    assert(partial->Map(0,nullptr,&mapped)==S_OK);
    for(unsigned i=0;i<1024;++i){const unsigned char expected=(i>=32&&i<288)?pattern(i-28):((i>=512&&i<640)?pattern(i-256):0xa5);assert(static_cast<unsigned char*>(mapped)[i]==expected);}partial->Unmap(0,nullptr);
    assert(complete->Map(0,nullptr,&mapped)==S_OK);for(unsigned i=0;i<1024;++i)assert(static_cast<unsigned char*>(mapped)[i]==pattern(i));complete->Unmap(0,nullptr);
    device.Reset();ComPtr<ID3D12Device> parent;assert(list->GetDevice(IID_PPV_ARGS(&parent))==S_OK);ComPtr<IUnknown> parent_identity;assert(parent.As(&parent_identity)==S_OK&&parent_identity.Get()==canonical);
    parent.Reset();assert(first->GetDevice(IID_PPV_ARGS(&parent))==S_OK);parent_identity.Reset();assert(parent.As(&parent_identity)==S_OK&&parent_identity.Get()==canonical);
    std::printf("{\"scenario\":\"command-copy-v1\",\"results\":[");for(size_t i=0;i<results.size();++i)std::printf("%s%u",i?",":"",results[i]);
    std::printf("],\"buffers\":[");hex(partial.Get());std::putchar(',');hex(complete.Get());std::puts("],\"deferred\":true,\"parent_lifetime\":true}");
}
