// Copyright (c) Alkyhol contributors. MIT licensed.
// Links the original pinned CommandListManager/CommandAllocatorPool objects.
// This supplies only a caller and the original global; it does not rewrite them.
#include <Windows.h>
#include <d3d12.h>
#include <wrl/client.h>
#include "CommandListManager.h"
#include <cassert>
#include <cstdio>
#include <cstring>
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
namespace Graphics { CommandListManager g_CommandManager; }
int main(){
    Microsoft::WRL::ComPtr<ID3D12Device> device;
    assert(AlkyholCreateMetalDeviceSubset(nullptr,IID_PPV_ARGS(&device))==S_OK);
    auto& manager=Graphics::g_CommandManager;
    manager.Create(device.Get());
    uint64_t final_values[3]={};unsigned index=0;
    for(auto type:{D3D12_COMMAND_LIST_TYPE_DIRECT,D3D12_COMMAND_LIST_TYPE_COMPUTE,D3D12_COMMAND_LIST_TYPE_COPY}){
        auto& queue=manager.GetQueue(type);assert(queue.IsReady());
        auto desc=queue.GetCommandQueue()->GetDesc();assert(desc.Type==type&&desc.NodeMask==1);
        for(unsigned i=1;i<=8;++i){auto fence=queue.IncrementFence();assert(fence==((uint64_t(type)<<56)|i));manager.WaitForFence(fence);assert(manager.IsFenceComplete(fence));}
        final_values[index++]=queue.GetNextFenceValue();queue.WaitForIdle();
    }
    unsigned copy_queues=0;
    for(auto type:{D3D12_COMMAND_LIST_TYPE_DIRECT,D3D12_COMMAND_LIST_TYPE_COMPUTE,D3D12_COMMAND_LIST_TYPE_COPY}){
        Microsoft::WRL::ComPtr<ID3D12GraphicsCommandList> list;ID3D12CommandAllocator* allocator=nullptr;
        manager.CreateNewCommandList(type,list.GetAddressOf(),&allocator);assert(list.Get()&&allocator&&list->GetType()==type);
        Microsoft::WRL::ComPtr<ID3D12Resource> source,destination;
        D3D12_HEAP_PROPERTIES heap={D3D12_HEAP_TYPE_UPLOAD,D3D12_CPU_PAGE_PROPERTY_UNKNOWN,D3D12_MEMORY_POOL_UNKNOWN,1,1};
        D3D12_RESOURCE_DESC desc={D3D12_RESOURCE_DIMENSION_BUFFER,0,257,1,1,1,DXGI_FORMAT_UNKNOWN,{1,0},D3D12_TEXTURE_LAYOUT_ROW_MAJOR,D3D12_RESOURCE_FLAG_NONE};
        assert(device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,D3D12_RESOURCE_STATE_GENERIC_READ,nullptr,IID_PPV_ARGS(&source))==S_OK);
        heap.Type=D3D12_HEAP_TYPE_READBACK;assert(device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,D3D12_RESOURCE_STATE_COPY_DEST,nullptr,IID_PPV_ARGS(&destination))==S_OK);
        void* mapped=nullptr;assert(source->Map(0,nullptr,&mapped)==S_OK);for(unsigned i=0;i<257;++i)static_cast<unsigned char*>(mapped)[i]=static_cast<unsigned char>(i*17+unsigned(type));source->Unmap(0,nullptr);
        assert(destination->Map(0,nullptr,&mapped)==S_OK);std::memset(mapped,0x5a,257);destination->Unmap(0,nullptr);
        list->CopyBufferRegion(destination.Get(),0,source.Get(),0,257);assert(list->Close()==S_OK);
        ID3D12CommandList* lists[]={list.Get()};auto& queue=manager.GetQueue(type);queue.GetCommandQueue()->ExecuteCommandLists(1,lists);manager.WaitForFence(queue.IncrementFence());
        assert(destination->Map(0,nullptr,&mapped)==S_OK);for(unsigned i=0;i<257;++i)assert(static_cast<unsigned char*>(mapped)[i]==static_cast<unsigned char>(i*17+unsigned(type)));destination->Unmap(0,nullptr);++copy_queues;
    }
    CommandAllocatorPool pool(D3D12_COMMAND_LIST_TYPE_COPY);pool.Create(device.Get());
    auto* first=pool.RequestAllocator(0);pool.DiscardAllocator(7,first);auto* second=pool.RequestAllocator(6);assert(second!=first);pool.DiscardAllocator(8,second);
    auto* reused=pool.RequestAllocator(7);assert(reused==first&&pool.Size()==2);pool.Shutdown();
    manager.IdleGPU();manager.Shutdown();assert(!manager.GetGraphicsQueue().IsReady()&&!manager.GetComputeQueue().IsReady()&&!manager.GetCopyQueue().IsReady());
    std::printf("{\"scenario\":\"original-command-manager-v3\",\"next_values\":[%llu,%llu,%llu],\"queues\":3,\"copy_queues\":%u,\"copy_bytes_per_queue\":257,\"allocator_pool_reused\":true,\"increments_per_queue\":8,\"idle\":true,\"shutdown\":true}\n",static_cast<unsigned long long>(final_values[0]),static_cast<unsigned long long>(final_values[1]),static_cast<unsigned long long>(final_values[2]),copy_queues);
}
