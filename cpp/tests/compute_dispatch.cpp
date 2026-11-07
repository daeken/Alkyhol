// Copyright (c) Alkyhol contributors. MIT licensed.
// Identical SDK/Rust COM dispatch workload, including buffer state transitions.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <array>
#include <vector>
#include <cassert>
#include <cstring>
#include <cstdio>
#include "compute_probe_dxil.h"
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
static ComPtr<ID3D12Resource> buffer(ID3D12Device* d,UINT64 width,D3D12_HEAP_TYPE kind,D3D12_RESOURCE_STATES state,D3D12_RESOURCE_FLAGS flags=D3D12_RESOURCE_FLAG_NONE){
    D3D12_HEAP_PROPERTIES heap={kind,D3D12_CPU_PAGE_PROPERTY_UNKNOWN,D3D12_MEMORY_POOL_UNKNOWN,1,1};D3D12_RESOURCE_DESC desc={D3D12_RESOURCE_DIMENSION_BUFFER,0,width,1,1,1,DXGI_FORMAT_UNKNOWN,{1,0},D3D12_TEXTURE_LAYOUT_ROW_MAJOR,flags};ComPtr<ID3D12Resource> r;assert(d->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,state,nullptr,IID_PPV_ARGS(&r))==S_OK);assert(r->GetDesc().Flags==flags);return r;
}
static void transition(ID3D12GraphicsCommandList* l,ID3D12Resource* r,D3D12_RESOURCE_STATES a,D3D12_RESOURCE_STATES b){D3D12_RESOURCE_BARRIER barrier{};barrier.Type=D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;barrier.Transition={r,D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,a,b};l->ResourceBarrier(1,&barrier);}
static void uav_barrier(ID3D12GraphicsCommandList* l,ID3D12Resource* r){D3D12_RESOURCE_BARRIER b{};b.Type=D3D12_RESOURCE_BARRIER_TYPE_UAV;b.UAV.pResource=r;l->ResourceBarrier(1,&b);}
static void write(ID3D12Resource* r,const std::vector<unsigned>& data,UINT offset=0){void* p=nullptr;assert(r->Map(0,nullptr,&p)==S_OK);std::memcpy(static_cast<unsigned char*>(p)+offset,data.data(),data.size()*4);r->Unmap(0,nullptr);}
static std::array<unsigned,104> read(ID3D12Resource* r){void* p=nullptr;assert(r->Map(0,nullptr,&p)==S_OK);std::array<unsigned,104> words{};std::memcpy(words.data(),p,sizeof(words));r->Unmap(0,nullptr);return words;}
static void bind(ID3D12GraphicsCommandList* l,ID3D12RootSignature* root,ID3D12Resource* cb,ID3D12Resource* a,ID3D12Resource* b,ID3D12Resource* out){l->SetComputeRootSignature(root);l->SetComputeRootConstantBufferView(0,cb->GetGPUVirtualAddress()+256);l->SetComputeRootShaderResourceView(1,a->GetGPUVirtualAddress()+16);l->SetComputeRootShaderResourceView(2,b->GetGPUVirtualAddress()+8);l->SetComputeRootUnorderedAccessView(3,out->GetGPUVirtualAddress()+16);}
static void execute(ID3D12CommandQueue* q,ID3D12GraphicsCommandList* l,ID3D12Fence* f,UINT64 value,HANDLE event){assert(l->Close()==S_OK);ID3D12CommandList* lists[]={l};q->ExecuteCommandLists(1,lists);assert(f->SetEventOnCompletion(value,event)==S_OK);assert(q->Signal(f,value)==S_OK);assert(WaitForSingleObject(event,10000)==WAIT_OBJECT_0);}
static void check(const std::array<unsigned,104>& words,unsigned variant){
    for(unsigned i=0;i<104;++i){unsigned expected=0xdeadbeefu;if(i>=4&&i<80){unsigned slot=(i-4)/4,component=(i-4)%4,source=slot/8*8+7-slot%8;unsigned a=source<19?(variant?1000u+source:0xfffffff0u+source):0,b=3*slot+7;float fp=4.f*float(slot)+22.f;unsigned bits=0;std::memcpy(&bits,&fp,4);unsigned values[]={a+0x12345678u,(b<<(slot&7))^0x12345678u,bits,(slot&1)?b:a};expected=values[component];}assert(words[i]==expected);}
}
int main(){
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    D3D12_ROOT_PARAMETER parameters[4]{};for(unsigned i=0;i<4;++i){parameters[i].ParameterType=i==0?D3D12_ROOT_PARAMETER_TYPE_CBV:i==3?D3D12_ROOT_PARAMETER_TYPE_UAV:D3D12_ROOT_PARAMETER_TYPE_SRV;parameters[i].Descriptor={0,i==2?2u:0u};}D3D12_ROOT_SIGNATURE_DESC root_desc={4,parameters,0,nullptr,D3D12_ROOT_SIGNATURE_FLAG_NONE};ComPtr<ID3DBlob> blob,error;assert(D3D12SerializeRootSignature(&root_desc,D3D_ROOT_SIGNATURE_VERSION_1,&blob,&error)==S_OK);ComPtr<ID3D12RootSignature> root;assert(device->CreateRootSignature(1,blob->GetBufferPointer(),blob->GetBufferSize(),IID_PPV_ARGS(&root))==S_OK);
    D3D12_COMPUTE_PIPELINE_STATE_DESC pd{};pd.pRootSignature=root.Get();pd.CS={compute_probe_dxil,sizeof(compute_probe_dxil)};pd.NodeMask=1;ComPtr<ID3D12PipelineState> first,second;assert(device->CreateComputePipelineState(&pd,IID_PPV_ARGS(&first))==S_OK);assert(device->CreateComputePipelineState(&pd,IID_PPV_ARGS(&second))==S_OK);
    auto cb=buffer(device.Get(),512,D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ);std::vector<unsigned> settings={19,0x12345678};for(float f:{2.f,.5f,1.f,1.f,1.f,1.f,2.f,3.f,4.f,5.f,-1.f,2.f,-3.f,4.f}){unsigned word=0;std::memcpy(&word,&f,4);settings.push_back(word);}write(cb.Get(),settings,256);
    auto a0=buffer(device.Get(),108,D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ),a1=buffer(device.Get(),108,D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ),b=buffer(device.Get(),108,D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ);std::vector<unsigned> av0,av1,bv;for(unsigned i=0;i<19;++i){av0.push_back(0xfffffff0u+i);av1.push_back(1000+i);bv.push_back(3*i+7);}write(a0.Get(),av0,16);write(a1.Get(),av1,16);write(b.Get(),bv,8);
    auto init=buffer(device.Get(),416,D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ);write(init.Get(),std::vector<unsigned>(104,0xdeadbeefu));std::vector<std::array<unsigned,104>> results;
    for(auto kind:{D3D12_COMMAND_LIST_TYPE_DIRECT,D3D12_COMMAND_LIST_TYPE_COMPUTE}){
        D3D12_COMMAND_QUEUE_DESC qd={kind,0,D3D12_COMMAND_QUEUE_FLAG_NONE,1};ComPtr<ID3D12CommandQueue> queue;ComPtr<ID3D12CommandAllocator> allocator;ComPtr<ID3D12GraphicsCommandList> list;assert(device->CreateCommandQueue(&qd,IID_PPV_ARGS(&queue))==S_OK);assert(device->CreateCommandAllocator(kind,IID_PPV_ARGS(&allocator))==S_OK);assert(device->CreateCommandList(1,kind,allocator.Get(),kind==D3D12_COMMAND_LIST_TYPE_DIRECT?first.Get():nullptr,IID_PPV_ARGS(&list))==S_OK);if(kind==D3D12_COMMAND_LIST_TYPE_COMPUTE)list->SetPipelineState(first.Get());
        auto out0=buffer(device.Get(),416,D3D12_HEAP_TYPE_DEFAULT,D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS),out1=buffer(device.Get(),416,D3D12_HEAP_TYPE_DEFAULT,D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS),read0=buffer(device.Get(),416,D3D12_HEAP_TYPE_READBACK,D3D12_RESOURCE_STATE_COPY_DEST),read1=buffer(device.Get(),416,D3D12_HEAP_TYPE_READBACK,D3D12_RESOURCE_STATE_COPY_DEST);
        list->CopyBufferRegion(out0.Get(),0,init.Get(),0,416);list->CopyBufferRegion(out1.Get(),0,init.Get(),0,416);list->CopyBufferRegion(out0.Get(),1,init.Get(),1,1);transition(list.Get(),out0.Get(),D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_STATE_UNORDERED_ACCESS);transition(list.Get(),out1.Get(),D3D12_RESOURCE_STATE_COPY_DEST,D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
        bind(list.Get(),root.Get(),cb.Get(),a0.Get(),b.Get(),out0.Get());list->Dispatch(3,1,1);uav_barrier(list.Get(),out0.Get());transition(list.Get(),out0.Get(),D3D12_RESOURCE_STATE_UNORDERED_ACCESS,D3D12_RESOURCE_STATE_COPY_SOURCE);list->CopyBufferRegion(read0.Get(),0,out0.Get(),0,416);list->CopyBufferRegion(read0.Get(),1,out0.Get(),1,1);
        list->SetComputeRootSignature(root.Get());list->SetPipelineState(second.Get());list->SetComputeRootShaderResourceView(1,a1->GetGPUVirtualAddress()+16);list->SetComputeRootUnorderedAccessView(3,out1->GetGPUVirtualAddress()+16);list->Dispatch(3,1,1);uav_barrier(list.Get(),out1.Get());transition(list.Get(),out1.Get(),D3D12_RESOURCE_STATE_UNORDERED_ACCESS,D3D12_RESOURCE_STATE_COPY_SOURCE);list->CopyBufferRegion(read1.Get(),0,out1.Get(),0,416);
        list->SetComputeRootShaderResourceView(1,a0->GetGPUVirtualAddress()+16);list->SetComputeRootUnorderedAccessView(3,out0->GetGPUVirtualAddress()+16);
        ComPtr<ID3D12Fence> fence;assert(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&fence))==S_OK);HANDLE event=CreateEventW(nullptr,FALSE,FALSE,nullptr);assert(event);execute(queue.Get(),list.Get(),fence.Get(),1,event);auto words0=read(read0.Get()),words1=read(read1.Get());check(words0,0);check(words1,1);results.push_back(words0);results.push_back(words1);
        assert(allocator->Reset()==S_OK);assert(list->Reset(allocator.Get(),second.Get())==S_OK);transition(list.Get(),out0.Get(),D3D12_RESOURCE_STATE_COMMON,D3D12_RESOURCE_STATE_UNORDERED_ACCESS);bind(list.Get(),root.Get(),cb.Get(),a0.Get(),b.Get(),out0.Get());list->Dispatch(3,1,1);uav_barrier(list.Get(),out0.Get());list->SetComputeRootShaderResourceView(1,a1->GetGPUVirtualAddress()+16);list->Dispatch(0,1,1);transition(list.Get(),out0.Get(),D3D12_RESOURCE_STATE_UNORDERED_ACCESS,D3D12_RESOURCE_STATE_COPY_SOURCE);list->CopyBufferRegion(read0.Get(),0,out0.Get(),0,416);execute(queue.Get(),list.Get(),fence.Get(),2,event);words0=read(read0.Get());check(words0,0);results.push_back(words0);assert(CloseHandle(event));
    }
    std::printf("{\"scenario\":\"compute-dispatch-v1\",\"cases\":[");for(size_t row=0;row<results.size();++row){if(row)std::putchar(',');std::putchar('[');for(size_t i=0;i<results[row].size();++i)std::printf("%s%u",i?",":"",results[row][i]);std::putchar(']');}std::puts("],\"queues\":2,\"root_offsets\":[256,16,8,16],\"reset\":true,\"zero_dispatch\":true,\"binding_snapshots\":true,\"same_root_preserved\":true}");
}
