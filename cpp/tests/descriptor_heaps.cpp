// Copyright (c) Alkyhol contributors. MIT licensed.
// Native SDK/WARP and explicit Metal heap allocation/identity/handle invariants.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdio>
#include <vector>
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
int main(){
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;
    assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    ComPtr<IUnknown> identity;assert(device.As(&identity)==S_OK);IUnknown* canonical=identity.Get();identity.Reset();factory.Reset();adapter.Reset();
    unsigned strides[4]={};for(unsigned i=0;i<4;++i){strides[i]=device->GetDescriptorHandleIncrementSize(static_cast<D3D12_DESCRIPTOR_HEAP_TYPE>(i));assert(strides[i]>0);}
    std::printf("{\"scenario\":\"descriptor-heaps-v1\",\"strides\":[%u,%u,%u,%u],\"heaps\":[",strides[0],strides[1],strides[2],strides[3]);bool first=true;
    std::vector<ComPtr<ID3D12DescriptorHeap>> heaps;
    for(unsigned type=0;type<4;++type)for(unsigned visible=0;visible<(type<2?2u:1u);++visible)for(unsigned node:{0u,1u}){
        D3D12_DESCRIPTOR_HEAP_DESC requested={static_cast<D3D12_DESCRIPTOR_HEAP_TYPE>(type),17,static_cast<D3D12_DESCRIPTOR_HEAP_FLAGS>(visible),node};
        assert(device->CreateDescriptorHeap(&requested,__uuidof(ID3D12DescriptorHeap),nullptr)==S_FALSE);
        ComPtr<ID3D12DescriptorHeap> heap;assert(device->CreateDescriptorHeap(&requested,IID_PPV_ARGS(&heap))==S_OK);assert(heap->SetName(L"descriptor heap test")==S_OK);
        ComPtr<ID3D12DeviceChild> child;ComPtr<ID3D12Pageable> pageable;ComPtr<IUnknown> own,child_id,pageable_id;assert(heap.As(&child)==S_OK&&heap.As(&pageable)==S_OK&&heap.As(&own)==S_OK&&child.As(&child_id)==S_OK&&pageable.As(&pageable_id)==S_OK);assert(own.Get()==child_id.Get()&&own.Get()==pageable_id.Get());
        ComPtr<ID3D12Device> parent;ComPtr<IUnknown> parent_id;assert(heap->GetDevice(IID_PPV_ARGS(&parent))==S_OK&&parent.As(&parent_id)==S_OK&&parent_id.Get()==canonical);
        const auto desc=heap->GetDesc();const auto cpu=heap->GetCPUDescriptorHandleForHeapStart();const auto gpu=heap->GetGPUDescriptorHandleForHeapStart();assert(cpu.ptr!=0);assert(heap->GetCPUDescriptorHandleForHeapStart().ptr==cpu.ptr&&heap->GetGPUDescriptorHandleForHeapStart().ptr==gpu.ptr);
        std::fprintf(stderr,"heap type=%u visible=%u node=%u returned_node=%u gpu=%016llx\n",type,visible,node,unsigned(desc.NodeMask),static_cast<unsigned long long>(gpu.ptr));
        // Addresses and hardware increments are opaque. Compare index offsets,
        // retaining raw strides separately instead of requiring vendor equality.
        const auto cpu_last=cpu.ptr+16*UINT64(strides[type]);const auto gpu_last=gpu.ptr+16*UINT64(strides[type]);assert((cpu_last-cpu.ptr)/strides[type]==16);if(visible)assert((gpu_last-gpu.ptr)/strides[type]==16);
        if(!first)std::putchar(',');first=false;std::printf("{\"desc\":[%u,%u,%u,%u],\"cpu_present\":true,\"gpu_present\":%s,\"gpu_invalid_sentinel\":%s,\"last_cpu_index\":%llu,\"last_gpu_index\":%llu}",unsigned(desc.Type),unsigned(desc.NumDescriptors),unsigned(desc.Flags),unsigned(desc.NodeMask),gpu.ptr?"true":"false",gpu.ptr==~UINT64(0)?"true":"false",static_cast<unsigned long long>((cpu_last-cpu.ptr)/strides[type]),static_cast<unsigned long long>(visible?(gpu_last-gpu.ptr)/strides[type]:0));heaps.push_back(heap);
    }
    device.Reset();ComPtr<ID3D12Device> parent;ComPtr<IUnknown> parent_id;assert(heaps.back()->GetDevice(IID_PPV_ARGS(&parent))==S_OK&&parent.As(&parent_id)==S_OK&&parent_id.Get()==canonical);heaps.clear();std::puts("],\"parent_lifetime\":true}");
}
