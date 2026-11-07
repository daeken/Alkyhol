// Copyright (c) Alkyhol contributors. MIT licensed.
// Shared CBV/copy workload. Native DXIL consumer vs explicitly bounded Metal probe.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <array>
#include <cassert>
#include <cstdio>
#include <cstring>
using Microsoft::WRL::ComPtr;
#ifdef _WIN32
#include "cbv_probe_dxil.h"
#else
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
extern "C" HRESULT AlkyholProbeCbvHeap(IUnknown*,UINT,UINT*,UINT);
#endif
static ComPtr<ID3D12Resource> buffer(ID3D12Device* device,UINT64 bytes,D3D12_HEAP_TYPE kind,D3D12_RESOURCE_STATES state,D3D12_RESOURCE_FLAGS flags=D3D12_RESOURCE_FLAG_NONE){
    D3D12_HEAP_PROPERTIES heap={kind,D3D12_CPU_PAGE_PROPERTY_UNKNOWN,D3D12_MEMORY_POOL_UNKNOWN,1,1};D3D12_RESOURCE_DESC desc={D3D12_RESOURCE_DIMENSION_BUFFER,0,bytes,1,1,1,DXGI_FORMAT_UNKNOWN,{1,0},D3D12_TEXTURE_LAYOUT_ROW_MAJOR,flags};ComPtr<ID3D12Resource> result;assert(device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,state,nullptr,IID_PPV_ARGS(&result))==S_OK);return result;
}
static unsigned value(unsigned block,unsigned word){return (block+1)*0x11111111u+word*0x01020304u;}
static D3D12_CPU_DESCRIPTOR_HANDLE at(ID3D12DescriptorHeap* heap,UINT stride,UINT index){auto handle=heap->GetCPUDescriptorHandleForHeapStart();handle.ptr+=UINT64(stride)*index;return handle;}
#ifdef _WIN32
static void barrier(ID3D12GraphicsCommandList* list,ID3D12Resource* r,D3D12_RESOURCE_STATES before,D3D12_RESOURCE_STATES after){D3D12_RESOURCE_BARRIER b{};b.Type=D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;b.Transition.pResource=r;b.Transition.Subresource=D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;b.Transition.StateBefore=before;b.Transition.StateAfter=after;list->ResourceBarrier(1,&b);}
#endif
static std::array<unsigned,24> consume(ID3D12Device* device,ID3D12DescriptorHeap* heap){
    std::array<unsigned,24> words;words.fill(0xcdcdcdcdu);
#ifdef _WIN32
    ComPtr<ID3D12RootSignature> root;assert(device->CreateRootSignature(0,cbv_probe_dxil,sizeof(cbv_probe_dxil),IID_PPV_ARGS(&root))==S_OK);D3D12_COMPUTE_PIPELINE_STATE_DESC pso{};pso.pRootSignature=root.Get();pso.CS={cbv_probe_dxil,sizeof(cbv_probe_dxil)};ComPtr<ID3D12PipelineState> pipeline;assert(device->CreateComputePipelineState(&pso,IID_PPV_ARGS(&pipeline))==S_OK);
    auto upload=buffer(device,96,D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ);auto output=buffer(device,96,D3D12_HEAP_TYPE_DEFAULT,D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS);auto readback=buffer(device,96,D3D12_HEAP_TYPE_READBACK,D3D12_RESOURCE_STATE_COPY_DEST);void* mapped=nullptr;assert(upload->Map(0,nullptr,&mapped)==S_OK);std::memcpy(mapped,words.data(),96);upload->Unmap(0,nullptr);
    ComPtr<ID3D12CommandAllocator> allocator;ComPtr<ID3D12GraphicsCommandList> list;assert(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT,IID_PPV_ARGS(&allocator))==S_OK);assert(device->CreateCommandList(0,D3D12_COMMAND_LIST_TYPE_DIRECT,allocator.Get(),pipeline.Get(),IID_PPV_ARGS(&list))==S_OK);list->CopyBufferRegion(output.Get(),0,upload.Get(),0,96);barrier(list.Get(),output.Get(),D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
    list->SetComputeRootSignature(root.Get());ID3D12DescriptorHeap* heaps[]={heap};list->SetDescriptorHeaps(1,heaps);list->SetComputeRootDescriptorTable(0,heap->GetGPUDescriptorHandleForHeapStart());list->SetComputeRootUnorderedAccessView(1,output->GetGPUVirtualAddress());list->Dispatch(1,1,1);barrier(list.Get(),output.Get(),D3D12_RESOURCE_STATE_UNORDERED_ACCESS,D3D12_RESOURCE_STATE_COPY_SOURCE);list->CopyBufferRegion(readback.Get(),0,output.Get(),0,96);assert(list->Close()==S_OK);
    D3D12_COMMAND_QUEUE_DESC desc{};ComPtr<ID3D12CommandQueue> queue;assert(device->CreateCommandQueue(&desc,IID_PPV_ARGS(&queue))==S_OK);ID3D12CommandList* lists[]={list.Get()};queue->ExecuteCommandLists(1,lists);ComPtr<ID3D12Fence> fence;assert(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&fence))==S_OK);HANDLE event=CreateEventW(nullptr,FALSE,FALSE,nullptr);assert(event);assert(fence->SetEventOnCompletion(1,event)==S_OK&&queue->Signal(fence.Get(),1)==S_OK);assert(WaitForSingleObject(event,5000)==WAIT_OBJECT_0);assert(CloseHandle(event));assert(readback->Map(0,nullptr,&mapped)==S_OK);std::memcpy(words.data(),mapped,96);readback->Unmap(0,nullptr);
#else
    (void)device;assert(AlkyholProbeCbvHeap(heap,0,words.data(),24)==S_OK);
#endif
    return words;
}
int main(){
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    auto constants=buffer(device.Get(),1024,D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ);void* mapped=nullptr;assert(constants->Map(0,nullptr,&mapped)==S_OK);for(unsigned block=0;block<4;++block)for(unsigned word=0;word<64;++word)static_cast<unsigned*>(mapped)[block*64+word]=value(block,word);constants->Unmap(0,nullptr);
    ComPtr<ID3D12DescriptorHeap> source,stage,visible;D3D12_DESCRIPTOR_HEAP_DESC desc={D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,8,D3D12_DESCRIPTOR_HEAP_FLAG_NONE,1};assert(device->CreateDescriptorHeap(&desc,IID_PPV_ARGS(&source))==S_OK);assert(device->CreateDescriptorHeap(&desc,IID_PPV_ARGS(&stage))==S_OK);desc.NumDescriptors=4;desc.Flags=D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE;assert(device->CreateDescriptorHeap(&desc,IID_PPV_ARGS(&visible))==S_OK);const auto stride=device->GetDescriptorHandleIncrementSize(desc.Type);
    for(unsigned block=0;block<4;++block){D3D12_CONSTANT_BUFFER_VIEW_DESC cbv={constants->GetGPUVirtualAddress()+block*256,256};device->CreateConstantBufferView(&cbv,at(source.Get(),stride,block));}
    device->CopyDescriptorsSimple(4,at(visible.Get(),stride,0),at(source.Get(),stride,0),desc.Type);std::array<std::array<unsigned,24>,3> results;results[0]=consume(device.Get(),visible.Get());
    D3D12_CONSTANT_BUFFER_VIEW_DESC changed={constants->GetGPUVirtualAddress()+768,256};device->CreateConstantBufferView(&changed,at(source.Get(),stride,0));results[1]=consume(device.Get(),visible.Get());
    const D3D12_CPU_DESCRIPTOR_HANDLE destinations[]={at(stage.Get(),stride,0),at(stage.Get(),stride,2)};const UINT destination_sizes[]={2,2};const D3D12_CPU_DESCRIPTOR_HANDLE sources[]={at(source.Get(),stride,0),at(source.Get(),stride,2),at(source.Get(),stride,1)};const UINT source_sizes[]={1,1,2};device->CopyDescriptors(2,destinations,destination_sizes,3,sources,source_sizes,desc.Type);
    D3D12_CPU_DESCRIPTOR_HANDLE ordered_dest[4],ordered_src[4];const unsigned order[]={3,0,1,2};for(unsigned i=0;i<4;++i){ordered_dest[i]=at(visible.Get(),stride,i);ordered_src[i]=at(stage.Get(),stride,order[i]);}device->CopyDescriptors(4,ordered_dest,nullptr,4,ordered_src,nullptr,desc.Type);results[2]=consume(device.Get(),visible.Get());
    const unsigned expected[3][4]={{0,1,2,3},{0,1,2,3},{2,3,2,1}};for(unsigned test=0;test<3;++test){for(unsigned slot=0;slot<4;++slot)for(unsigned word=0;word<4;++word)assert(results[test][slot*4+word]==value(expected[test][slot],word));for(unsigned tail=16;tail<24;++tail)assert(results[test][tail]==0xcdcdcdcdu);}
    std::printf("{\"scenario\":\"cbv-copy-v1\",\"cases\":[");for(unsigned test=0;test<3;++test){if(test)std::putchar(',');std::putchar('[');for(unsigned i=0;i<24;++i)std::printf("%s%u",i?",":"",results[test][i]);std::putchar(']');}std::puts("],\"snapshot_independent\":true,\"gpu_consumed\":true}");
}
