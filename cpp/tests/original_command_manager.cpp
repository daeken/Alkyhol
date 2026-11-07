// Copyright (c) Alkyhol contributors. MIT licensed.
// Links original queue, command-allocator and descriptor-heap objects.
// This supplies only a caller and matching original globals; it does not rewrite them.
#include <Windows.h>
#include <d3d12.h>
#include <wrl/client.h>
#include "CommandListManager.h"
// Same public constant as the pinned MiniEngine pch.h.
#define D3D12_GPU_VIRTUAL_ADDRESS_UNKNOWN ((D3D12_GPU_VIRTUAL_ADDRESS)-1)
#include "DescriptorHeap.h"
#include "RootSignature.h"
#include <cassert>
#include <cstdio>
#include <cstring>
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
namespace Graphics { CommandListManager g_CommandManager; ID3D12Device* g_Device=nullptr; }
int main(){
    Microsoft::WRL::ComPtr<ID3D12Device> device;
    assert(AlkyholCreateMetalDeviceSubset(nullptr,IID_PPV_ARGS(&device))==S_OK);
    Graphics::g_Device=device.Get();
    {
        RootSignature first(1),cached(1),mixed(5,1);
        first[0].InitAsDescriptorRange(D3D12_DESCRIPTOR_RANGE_TYPE_SRV,3,4,D3D12_SHADER_VISIBILITY_PIXEL,2);
        first.Finalize(L"unchanged original root helper");auto* identity=first.GetSignature();assert(identity);
        first.Finalize(L"already finalized");assert(first.GetSignature()==identity);
        cached[0].InitAsDescriptorRange(D3D12_DESCRIPTOR_RANGE_TYPE_SRV,3,4,D3D12_SHADER_VISIBILITY_PIXEL,2);
        cached.Finalize(L"original cached root helper");assert(cached.GetSignature()==identity);
        mixed[0].InitAsConstants(0,4);mixed[1].InitAsConstantBuffer(1);mixed[2].InitAsBufferSRV(3);mixed[3].InitAsBufferUAV(2);
        mixed[4].InitAsDescriptorTable(2,D3D12_SHADER_VISIBILITY_PIXEL);mixed[4].SetTableRange(0,D3D12_DESCRIPTOR_RANGE_TYPE_SRV,0,2,1);mixed[4].SetTableRange(1,D3D12_DESCRIPTOR_RANGE_TYPE_CBV,0,3,1);
        D3D12_SAMPLER_DESC sampler{};sampler.Filter=D3D12_FILTER_MIN_MAG_MIP_LINEAR;sampler.AddressU=sampler.AddressV=sampler.AddressW=D3D12_TEXTURE_ADDRESS_MODE_WRAP;sampler.MaxAnisotropy=1;sampler.ComparisonFunc=D3D12_COMPARISON_FUNC_ALWAYS;sampler.MaxLOD=FLT_MAX;
        mixed.InitStaticSampler(0,sampler,D3D12_SHADER_VISIBILITY_PIXEL);mixed.Finalize(L"original mixed root helper",D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT);assert(mixed.GetSignature());
        Microsoft::WRL::ComPtr<ID3D12Device> parent;assert(mixed.GetSignature()->GetDevice(IID_PPV_ARGS(&parent))==S_OK&&parent.Get()==device.Get());
        RootSignature::DestroyAll();
    }
    unsigned descriptor_types=0,visible_heaps=0;
    for(auto type:{D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,D3D12_DESCRIPTOR_HEAP_TYPE_RTV,D3D12_DESCRIPTOR_HEAP_TYPE_DSV}) {
        DescriptorAllocator descriptors(type);const auto first=descriptors.Allocate(128);const auto second=descriptors.Allocate(128);const auto rollover=descriptors.Allocate(1);const auto stride=device->GetDescriptorHandleIncrementSize(type);
        assert(first.ptr&&second.ptr==first.ptr+128*UINT64(stride));assert(rollover.ptr<first.ptr||rollover.ptr>=first.ptr+256*UINT64(stride));++descriptor_types;
    }
    for(auto type:{D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER}) {
        DescriptorHeap heap;heap.Create(L"original descriptor heap",type,16);const auto first=heap.Alloc(3);const auto second=heap.Alloc(5);assert(heap.GetOffsetOfHandle(first)==0&&heap.GetOffsetOfHandle(second)==3);assert(heap.ValidateHandle(first)&&heap.ValidateHandle(second)&&heap.ValidateHandle(heap[15]));assert(!heap.ValidateHandle(heap[16]));
        assert(heap.HasAvailableSpace(8)&&!heap.HasAvailableSpace(9));heap.Alloc(8);assert(!heap.HasAvailableSpace(1));heap.Destroy();++visible_heaps;
    }
    DescriptorAllocator::DestroyAll();
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
    std::printf("{\"scenario\":\"original-command-manager-v5\",\"next_values\":[%llu,%llu,%llu],\"queues\":3,\"copy_queues\":%u,\"copy_bytes_per_queue\":257,\"descriptor_types\":%u,\"visible_heaps\":%u,\"root_signatures\":3,\"root_cache_reused\":true,\"allocator_pool_reused\":true,\"increments_per_queue\":8,\"idle\":true,\"shutdown\":true}\n",static_cast<unsigned long long>(final_values[0]),static_cast<unsigned long long>(final_values[1]),static_cast<unsigned long long>(final_values[2]),copy_queues,descriptor_types,visible_heaps);
    Graphics::g_Device=nullptr;
}
