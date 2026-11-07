// Copyright (c) Alkyhol contributors. MIT licensed.
// Same descriptor/map/ownership sequence on native WARP and a restricted Metal
// constructor. The latter does not claim D3D12CreateDevice/feature-level support.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <string>
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
int main(int argc,char** argv){
#ifdef _WIN32
    (void)argc;(void)argv;
#endif
    ComPtr<IDXGIFactory4> factory;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
    ComPtr<IDXGIAdapter1> adapter;
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);
#endif
    DXGI_ADAPTER_DESC1 adapter_desc{};assert(adapter->GetDesc1(&adapter_desc)==S_OK);
#ifdef _WIN32
    assert(adapter_desc.Flags&DXGI_ADAPTER_FLAG_SOFTWARE);
    std::fprintf(stderr,"adapter: native D3D12 WARP vendor=%u device=%u flags=%u\n",unsigned(adapter_desc.VendorId),unsigned(adapter_desc.DeviceId),unsigned(adapter_desc.Flags));
#endif
    ComPtr<ID3D12Device> device;
#ifdef _WIN32
    assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    assert(device->GetNodeCount()==1);assert(device->SetName(L"buffer test device")==S_OK);
    const LUID luid=device->GetAdapterLuid();assert(luid.LowPart==adapter_desc.AdapterLuid.LowPart&&luid.HighPart==adapter_desc.AdapterLuid.HighPart);
    ComPtr<IUnknown> canonical;assert(device.As(&canonical)==S_OK);
    ComPtr<ID3D12Device> via_unknown;assert(canonical.As(&via_unknown)==S_OK);ComPtr<IUnknown> roundtrip_unknown;assert(via_unknown.As(&roundtrip_unknown)==S_OK&&roundtrip_unknown.Get()==canonical.Get());
    IUnknown* device_identity=canonical.Get();roundtrip_unknown.Reset();via_unknown.Reset();canonical.Reset();factory.Reset();adapter.Reset();
#ifndef _WIN32
    if(argc==2){const std::string mode=argv[1];
        if(mode=="trap-view")device->CreateConstantBufferView(nullptr,{0});
        if(mode=="trap-heap")device->GetCustomHeapProperties(1,D3D12_HEAP_TYPE_UPLOAD);
        if(mode=="trap-copy")device->CopyDescriptorsSimple(1,{0},{0},D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
        if(mode=="trap-footprints")device->GetCopyableFootprints(nullptr,0,1,0,nullptr,nullptr,nullptr,nullptr);
        if(mode=="trap-tiling")device->GetResourceTiling(nullptr,nullptr,nullptr,nullptr,nullptr,0,nullptr);
        return 99; // Returning is a failed unsupported-operation test.
    }
    D3D12_FEATURE_DATA_D3D12_OPTIONS5 options{};assert(device->CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS5,&options,sizeof(options))==S_OK&&options.RaytracingTier==D3D12_RAYTRACING_TIER_NOT_SUPPORTED);
    ComPtr<ID3D12Device5> dxr;assert(device.As(&dxr)==E_NOINTERFACE&&!dxr.Get());
    ComPtr<ID3D12CommandQueue> queue;D3D12_COMMAND_QUEUE_DESC queue_desc{};assert(device->CreateCommandQueue(&queue_desc,IID_PPV_ARGS(&queue))==E_NOTIMPL&&!queue.Get());
#endif
    std::printf("{\"scenario\":\"device-shared-buffers-v1\",\"resources\":[");bool first=true;
    ComPtr<ID3D12Resource> retained;
    for(auto type:{D3D12_HEAP_TYPE_UPLOAD,D3D12_HEAP_TYPE_READBACK})for(uint64_t width:{uint64_t(257),uint64_t(1024)}){
        D3D12_HEAP_PROPERTIES heap={type,D3D12_CPU_PAGE_PROPERTY_UNKNOWN,D3D12_MEMORY_POOL_UNKNOWN,1,1};
        D3D12_RESOURCE_DESC desc={D3D12_RESOURCE_DIMENSION_BUFFER,0,width,1,1,1,DXGI_FORMAT_UNKNOWN,{1,0},D3D12_TEXTURE_LAYOUT_ROW_MAJOR,D3D12_RESOURCE_FLAG_NONE};
        D3D12_RESOURCE_STATES state=type==D3D12_HEAP_TYPE_UPLOAD?D3D12_RESOURCE_STATE_GENERIC_READ:D3D12_RESOURCE_STATE_COPY_DEST;
        ComPtr<ID3D12Resource> resource;assert(device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,state,nullptr,IID_PPV_ARGS(&resource))==S_OK);
        ComPtr<ID3D12Device> parent;assert(resource->GetDevice(IID_PPV_ARGS(&parent))==S_OK);ComPtr<IUnknown> parent_identity;assert(parent.As(&parent_identity)==S_OK&&parent_identity.Get()==device_identity);
        ComPtr<ID3D12DeviceChild> child;ComPtr<ID3D12Pageable> pageable;assert(resource.As(&child)==S_OK&&resource.As(&pageable)==S_OK);
        ComPtr<IUnknown> resource_identity,child_identity,pageable_identity;assert(resource.As(&resource_identity)==S_OK&&child.As(&child_identity)==S_OK&&pageable.As(&pageable_identity)==S_OK);
        assert(resource_identity.Get()==child_identity.Get()&&resource_identity.Get()==pageable_identity.Get());
        assert(resource->SetName(L"shared buffer")==S_OK);
        const auto actual=resource->GetDesc();assert(actual.Width==width&&actual.Dimension==D3D12_RESOURCE_DIMENSION_BUFFER);
        D3D12_HEAP_PROPERTIES properties{};D3D12_HEAP_FLAGS flags{};assert(resource->GetHeapProperties(&properties,&flags)==S_OK);
        void* mapped=nullptr;D3D12_RANGE no_read={0,0};assert(resource->Map(0,&no_read,&mapped)==S_OK&&mapped);auto* bytes=static_cast<unsigned char*>(mapped);
        for(uint64_t i=0;i<width;++i)bytes[i]=static_cast<unsigned char>((i*17+3)&255);D3D12_RANGE written={0,size_t(width)};resource->Unmap(0,&written);
        D3D12_RANGE read={0,size_t(width)};assert(resource->Map(0,&read,&mapped)==S_OK);uint32_t checksum=2166136261u;
        bytes=static_cast<unsigned char*>(mapped);for(uint64_t i=0;i<width;++i){assert(bytes[i]==static_cast<unsigned char>((i*17+3)&255));checksum=(checksum^bytes[i])*16777619u;}resource->Unmap(0,nullptr);
        assert(resource->GetGPUVirtualAddress()!=0);
        if(!first)std::putchar(',');first=false;
        std::printf("{\"heap\":[%u,%u,%u,%u,%u],\"heap_flags\":%u,\"desc\":[%u,%llu,%llu,%u,%u,%u,%u,%u,%u,%u,%u],\"checksum\":%u}",unsigned(properties.Type),unsigned(properties.CPUPageProperty),unsigned(properties.MemoryPoolPreference),unsigned(properties.CreationNodeMask),unsigned(properties.VisibleNodeMask),unsigned(flags),unsigned(actual.Dimension),static_cast<unsigned long long>(actual.Alignment),static_cast<unsigned long long>(actual.Width),unsigned(actual.Height),unsigned(actual.DepthOrArraySize),unsigned(actual.MipLevels),unsigned(actual.Format),unsigned(actual.SampleDesc.Count),unsigned(actual.SampleDesc.Quality),unsigned(actual.Layout),unsigned(actual.Flags),unsigned(checksum));
        retained=resource;
    }
    device.Reset();ComPtr<ID3D12Device> parent;assert(retained->GetDevice(IID_PPV_ARGS(&parent))==S_OK);ComPtr<IUnknown> final_identity;assert(parent.As(&final_identity)==S_OK&&final_identity.Get()==device_identity);retained.Reset();
    std::puts("],\"parent_lifetime\":true,\"gpu_addresses_nonzero\":true}");
}
