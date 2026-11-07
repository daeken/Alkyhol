// Copyright (c) Alkyhol contributors. MIT licensed.
// Finite unaligned-copy workload shared verbatim by native WARP and Metal.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <vector>
using Microsoft::WRL::ComPtr;
static const unsigned chunk=16*1024*1024;
static const unsigned lengths[]={1,2,3,4,5,6,7,15,16,17,31,32,33,255,256,257,1023};
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
static unsigned char pattern(unsigned i){return static_cast<unsigned char>(((i*23+11)^(i>>7)^(i>>15)^(i>>23))&255);}
static ComPtr<ID3D12Resource> buffer(ID3D12Device* device,D3D12_HEAP_TYPE type,UINT64 width){
    ComPtr<ID3D12Resource> result;D3D12_HEAP_PROPERTIES heap={type,D3D12_CPU_PAGE_PROPERTY_UNKNOWN,D3D12_MEMORY_POOL_UNKNOWN,1,1};D3D12_RESOURCE_DESC desc={D3D12_RESOURCE_DIMENSION_BUFFER,0,width,1,1,1,DXGI_FORMAT_UNKNOWN,{1,0},D3D12_TEXTURE_LAYOUT_ROW_MAJOR,D3D12_RESOURCE_FLAG_NONE};
    assert(device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,type==D3D12_HEAP_TYPE_UPLOAD?D3D12_RESOURCE_STATE_GENERIC_READ:(type==D3D12_HEAP_TYPE_DEFAULT?D3D12_RESOURCE_STATE_COMMON:D3D12_RESOURCE_STATE_COPY_DEST),nullptr,IID_PPV_ARGS(&result))==S_OK);return result;
}
static void transition(ID3D12GraphicsCommandList* list,ID3D12Resource* resource){D3D12_RESOURCE_BARRIER barrier{};barrier.Type=D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;barrier.Transition.pResource=resource;barrier.Transition.Subresource=D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;barrier.Transition.StateBefore=D3D12_RESOURCE_STATE_COPY_DEST;barrier.Transition.StateAfter=D3D12_RESOURCE_STATE_COPY_SOURCE;list->ResourceBarrier(1,&barrier);}
static void fill(ID3D12Resource* resource,unsigned length,bool source){void* p=nullptr;assert(resource->Map(0,nullptr,&p)==S_OK);for(unsigned i=0;i<length;++i)static_cast<unsigned char*>(p)[i]=source?pattern(i):0xa5;resource->Unmap(0,nullptr);}
static void hex(const unsigned char* bytes,unsigned length){std::putchar('"');for(unsigned i=0;i<length;++i)std::printf("%02x",unsigned(bytes[i]));std::putchar('"');}
int main(){
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    std::vector<unsigned char> expected(16,0xa5);struct Region{unsigned source,destination,length;};std::vector<Region> regions;
    for(unsigned source=0;source<4;++source)for(unsigned destination=0;destination<4;++destination)for(unsigned length:lengths){const unsigned base=(static_cast<unsigned>(expected.size())+3)&~3u;const unsigned offset=base+4+destination;expected.resize(offset+length+8,0xa5);for(unsigned i=0;i<length;++i)expected[offset+i]=pattern(source+i);regions.push_back({source,offset,length});}
    const unsigned count=static_cast<unsigned>(expected.size());const unsigned width=4099;const unsigned large_width=chunk+97;const unsigned large_length=chunk+19;
    auto upload=buffer(device.Get(),D3D12_HEAP_TYPE_UPLOAD,width);auto private_buffer=buffer(device.Get(),D3D12_HEAP_TYPE_DEFAULT,width);auto overlay=buffer(device.Get(),D3D12_HEAP_TYPE_DEFAULT,width);auto readback=buffer(device.Get(),D3D12_HEAP_TYPE_READBACK,count);auto overlay_readback=buffer(device.Get(),D3D12_HEAP_TYPE_READBACK,width);auto large_source=buffer(device.Get(),D3D12_HEAP_TYPE_UPLOAD,large_width);auto large_readback=buffer(device.Get(),D3D12_HEAP_TYPE_READBACK,large_width);
    fill(upload.Get(),width,true);fill(readback.Get(),count,false);fill(overlay_readback.Get(),width,false);fill(large_source.Get(),large_width,true);fill(large_readback.Get(),large_width,false);
    ComPtr<ID3D12CommandAllocator> allocator;ComPtr<ID3D12GraphicsCommandList> list;assert(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT,IID_PPV_ARGS(&allocator))==S_OK);assert(device->CreateCommandList(1,D3D12_COMMAND_LIST_TYPE_DIRECT,allocator.Get(),nullptr,IID_PPV_ARGS(&list))==S_OK);
    // Odd full-resource upload, then mixed compute/blit reads from Private.
    list->CopyResource(private_buffer.Get(),upload.Get());transition(list.Get(),private_buffer.Get());for(auto region:regions)list->CopyBufferRegion(readback.Get(),region.destination,private_buffer.Get(),region.source,region.length);
    // Adjacent/overlapping byte writes must preserve all bytes outside each range.
    list->CopyResource(overlay.Get(),upload.Get());std::vector<unsigned char> overlay_expected(width);for(unsigned i=0;i<width;++i)overlay_expected[i]=pattern(i);
    for(auto region:std::vector<Region>{{3,1,1},{7,2,3},{11,5,7},{1,4095,4},{9,3,5}}){list->CopyBufferRegion(overlay.Get(),region.destination,upload.Get(),region.source,region.length);for(unsigned i=0;i<region.length;++i)overlay_expected[region.destination+i]=pattern(region.source+i);}
    transition(list.Get(),overlay.Get());list->CopyResource(overlay_readback.Get(),overlay.Get());list->CopyBufferRegion(large_readback.Get(),5,large_source.Get(),3,large_length);assert(list->Close()==S_OK);
    void* mapped=nullptr;assert(readback->Map(0,nullptr,&mapped)==S_OK);for(unsigned i=0;i<count;++i)assert(static_cast<unsigned char*>(mapped)[i]==0xa5);readback->Unmap(0,nullptr);
    D3D12_COMMAND_QUEUE_DESC desc={D3D12_COMMAND_LIST_TYPE_DIRECT,0,D3D12_COMMAND_QUEUE_FLAG_NONE,1};ComPtr<ID3D12CommandQueue> queue;assert(device->CreateCommandQueue(&desc,IID_PPV_ARGS(&queue))==S_OK);ID3D12CommandList* lists[]={list.Get()};queue->ExecuteCommandLists(1,lists);ComPtr<ID3D12Fence> fence;assert(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&fence))==S_OK);HANDLE event=CreateEventW(nullptr,FALSE,FALSE,nullptr);assert(event);assert(fence->SetEventOnCompletion(1,event)==S_OK&&queue->Signal(fence.Get(),1)==S_OK);assert(WaitForSingleObject(event,15000)==WAIT_OBJECT_0);assert(CloseHandle(event));
    std::printf("{\"scenario\":\"byte-copies-v1\",\"cases\":%u,\"residue_buffer\":",unsigned(regions.size()));assert(readback->Map(0,nullptr,&mapped)==S_OK);assert(!std::memcmp(mapped,expected.data(),count));hex(static_cast<unsigned char*>(mapped),count);readback->Unmap(0,nullptr);
    std::printf(",\"overlay_buffer\":");assert(overlay_readback->Map(0,nullptr,&mapped)==S_OK);assert(!std::memcmp(mapped,overlay_expected.data(),width));hex(static_cast<unsigned char*>(mapped),width);overlay_readback->Unmap(0,nullptr);
    assert(large_readback->Map(0,nullptr,&mapped)==S_OK);const auto* large=static_cast<const unsigned char*>(mapped);unsigned checksum=2166136261u;for(unsigned i=0;i<large_width;++i){const unsigned char value=(i>=5&&i<5+large_length)?pattern(i-2):0xa5;assert(large[i]==value);checksum=(checksum^large[i])*16777619u;}
    std::printf(",\"large_bytes_checked\":%u,\"large_checksum\":%u,\"large_windows\":[",large_width,checksum);hex(large,32);std::putchar(',');hex(large+chunk-16,64);std::putchar(',');hex(large+large_width-32,32);large_readback->Unmap(0,nullptr);std::puts("],\"deferred\":true}");
}
