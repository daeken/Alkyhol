// Copyright (c) Alkyhol contributors. MIT licensed.
// Same PSO construction/COM workload against native SDK or restricted Rust.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <vector>
#include <algorithm>
#include "compute_probe_dxil.h"
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
int main(){
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    D3D12_ROOT_PARAMETER parameters[4]{};
    for(unsigned i=0;i<4;++i){parameters[i].ParameterType=i==0?D3D12_ROOT_PARAMETER_TYPE_CBV:i==3?D3D12_ROOT_PARAMETER_TYPE_UAV:D3D12_ROOT_PARAMETER_TYPE_SRV;parameters[i].Descriptor={0,i==2?2u:0u};}
    D3D12_ROOT_SIGNATURE_DESC root_desc={4,parameters,0,nullptr,D3D12_ROOT_SIGNATURE_FLAG_NONE};ComPtr<ID3DBlob> blob,error;assert(D3D12SerializeRootSignature(&root_desc,D3D_ROOT_SIGNATURE_VERSION_1,&blob,&error)==S_OK);
    ComPtr<ID3D12RootSignature> root;assert(device->CreateRootSignature(1,blob->GetBufferPointer(),blob->GetBufferSize(),IID_PPV_ARGS(&root))==S_OK);
    std::vector<unsigned char> bytecode(compute_probe_dxil,compute_probe_dxil+sizeof(compute_probe_dxil));D3D12_COMPUTE_PIPELINE_STATE_DESC desc{};desc.pRootSignature=root.Get();desc.CS={bytecode.data(),bytecode.size()};desc.NodeMask=1;
    assert(device->CreateComputePipelineState(&desc,__uuidof(ID3D12PipelineState),nullptr)==S_FALSE);
    ComPtr<ID3D12PipelineState> pipeline,second;assert(device->CreateComputePipelineState(&desc,IID_PPV_ARGS(&pipeline))==S_OK);assert(device->CreateComputePipelineState(&desc,IID_PPV_ARGS(&second))==S_OK);second.Reset();
    std::fill(bytecode.begin(),bytecode.end(),0);root.Reset();blob.Reset();device.Reset();
    ComPtr<ID3D12Device> parent;assert(pipeline->GetDevice(IID_PPV_ARGS(&parent))==S_OK&&parent->GetNodeCount()==1);
    ComPtr<IUnknown> identity,again;assert(pipeline->QueryInterface(IID_PPV_ARGS(&identity))==S_OK);assert(identity->QueryInterface(IID_PPV_ARGS(&again))==S_OK&&identity.Get()==again.Get());
    ComPtr<ID3D12Pageable> pageable;assert(pipeline->QueryInterface(IID_PPV_ARGS(&pageable))==S_OK);
    assert(pipeline->SetName(L"open DXIL compute pipeline \U0001f338")==S_OK);const GUID key={0x79170844,1,2,{1,2,3,4,5,6,7,8}};
    const unsigned input[]={0xfffedcba,0x1701,0x8192};unsigned output[3]={};UINT size=sizeof(output);assert(pipeline->SetPrivateData(key,sizeof(input),input)==S_OK);assert(pipeline->GetPrivateData(key,&size,output)==S_OK&&size==sizeof(output)&&std::memcmp(input,output,size)==0);
#ifndef _WIN32
    // Windows cache formats are not implemented; no fabricated successful blob.
    ComPtr<ID3DBlob> cache;assert(pipeline->GetCachedBlob(&cache)==E_NOTIMPL&&!cache);
#endif
    std::puts("{\"scenario\":\"compute-pso-v1\",\"created\":2,\"capability_checked\":true,\"canonical_identity\":true,\"pageable\":true,\"metadata\":true,\"parent_lifetime\":true}");
}
