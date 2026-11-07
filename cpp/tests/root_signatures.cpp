// Copyright (c) Alkyhol contributors. MIT licensed.
// Same source against the independent Windows SDK or restricted Rust COM.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <array>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <vector>
#include <cfloat>
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
static const GUID tag={0x71e1c044,0x1932,0x4021,{1,2,3,4,5,6,7,8}};
static std::vector<unsigned char> capture(ID3D12Device* device,const D3D12_ROOT_SIGNATURE_DESC& desc,ComPtr<ID3D12RootSignature>& retained){
    ComPtr<ID3DBlob> blob,error;
    auto hr=D3D12SerializeRootSignature(&desc,D3D_ROOT_SIGNATURE_VERSION_1,&blob,&error);
    if(FAILED(hr))std::fprintf(stderr,"Root serialization failed 0x%08x: %s\n",unsigned(hr),error?static_cast<const char*>(error->GetBufferPointer()):"no diagnostic");
    assert(hr==S_OK&&blob&&!error);
    const auto* begin=static_cast<const unsigned char*>(blob->GetBufferPointer());std::vector<unsigned char> bytes(begin,begin+blob->GetBufferSize());
    D3D12_VERSIONED_ROOT_SIGNATURE_DESC versioned{};versioned.Version=D3D_ROOT_SIGNATURE_VERSION_1_0;versioned.Desc_1_0=desc;ComPtr<ID3DBlob> other;
    assert(D3D12SerializeVersionedRootSignature(&versioned,&other,&error)==S_OK&&!error);
    assert(other->GetBufferSize()==bytes.size()&&std::memcmp(other->GetBufferPointer(),bytes.data(),bytes.size())==0);
    assert(device->CreateRootSignature(0,bytes.data(),bytes.size(),__uuidof(ID3D12RootSignature),nullptr)==S_FALSE);
    assert(device->CreateRootSignature(1,bytes.data(),bytes.size(),IID_PPV_ARGS(&retained))==S_OK);
    ComPtr<IUnknown> identity,again;assert(retained->QueryInterface(IID_PPV_ARGS(&identity))==S_OK);assert(identity->QueryInterface(IID_PPV_ARGS(&again))==S_OK&&identity.Get()==again.Get());
    ComPtr<ID3D12Device> parent;assert(retained->GetDevice(IID_PPV_ARGS(&parent))==S_OK);ComPtr<IUnknown> parent_identity,device_identity;assert(parent->QueryInterface(IID_PPV_ARGS(&parent_identity))==S_OK&&device->QueryInterface(IID_PPV_ARGS(&device_identity))==S_OK&&parent_identity.Get()==device_identity.Get());
    ComPtr<ID3D12Pageable> pageable;assert(retained->QueryInterface(IID_PPV_ARGS(&pageable))==E_NOINTERFACE&&!pageable);
    assert(retained->SetName(L"original root layout \U0001f338")==S_OK);
    const unsigned input[]={0x17u,0xfedcba98u,0x10203040u};assert(retained->SetPrivateData(tag,sizeof(input),input)==S_OK);unsigned actual[3]={};UINT size=sizeof(actual);assert(retained->GetPrivateData(tag,&size,actual)==S_OK&&size==sizeof(actual)&&std::memcmp(input,actual,sizeof(input))==0);
    // The created object must own its layout, independently of serializer blobs.
    std::memset(blob->GetBufferPointer(),0,blob->GetBufferSize());other.Reset();blob.Reset();
    return bytes;
}
int main(){
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    std::vector<std::vector<unsigned char>> layouts;ComPtr<ID3D12RootSignature> retained;D3D12_ROOT_SIGNATURE_DESC desc{};
    layouts.push_back(capture(device.Get(),desc,retained));
    D3D12_ROOT_PARAMETER constants{};constants.ParameterType=D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS;constants.Constants={3,2,64};desc.NumParameters=1;desc.pParameters=&constants;
    layouts.push_back(capture(device.Get(),desc,retained));
    std::array<D3D12_ROOT_PARAMETER,7> descriptors{};for(unsigned i=0;i<7;++i){descriptors[i].ParameterType=static_cast<D3D12_ROOT_PARAMETER_TYPE>(2+i%3);descriptors[i].Descriptor={i,2};}desc.NumParameters=7;desc.pParameters=descriptors.data();
    layouts.push_back(capture(device.Get(),desc,retained));
    D3D12_DESCRIPTOR_RANGE ranges[]={{D3D12_DESCRIPTOR_RANGE_TYPE_SRV,3,0,0,D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND},{D3D12_DESCRIPTOR_RANGE_TYPE_CBV,2,1,0,8},{D3D12_DESCRIPTOR_RANGE_TYPE_UAV,2,4,0,D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND}};
    D3D12_DESCRIPTOR_RANGE sampler_range={D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER,4,4,0,D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND};D3D12_ROOT_PARAMETER tables[2]{};tables[0].ParameterType=D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE;tables[0].DescriptorTable={3,ranges};tables[1].ParameterType=D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE;tables[1].DescriptorTable={1,&sampler_range};tables[1].ShaderVisibility=D3D12_SHADER_VISIBILITY_PIXEL;desc.NumParameters=2;desc.pParameters=tables;
    layouts.push_back(capture(device.Get(),desc,retained));
    D3D12_STATIC_SAMPLER_DESC samplers[2]{};for(unsigned i=0;i<2;++i){auto& s=samplers[i];s.Filter=i?D3D12_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR:D3D12_FILTER_MIN_MAG_MIP_LINEAR;s.AddressU=D3D12_TEXTURE_ADDRESS_MODE_WRAP;s.AddressV=D3D12_TEXTURE_ADDRESS_MODE_CLAMP;s.AddressW=D3D12_TEXTURE_ADDRESS_MODE_BORDER;s.MaxAnisotropy=16;s.ComparisonFunc=D3D12_COMPARISON_FUNC_LESS_EQUAL;s.BorderColor=D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE;s.MaxLOD=FLT_MAX;s.ShaderRegister=i;s.ShaderVisibility=D3D12_SHADER_VISIBILITY_PIXEL;}desc.NumStaticSamplers=2;desc.pStaticSamplers=samplers;
    layouts.push_back(capture(device.Get(),desc,retained));
    D3D12_ROOT_PARAMETER mixed[3]{};mixed[0].ParameterType=D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS;mixed[0].Constants={0,2,8};mixed[0].ShaderVisibility=D3D12_SHADER_VISIBILITY_VERTEX;mixed[1].ParameterType=D3D12_ROOT_PARAMETER_TYPE_CBV;mixed[1].Descriptor={0,2};mixed[1].ShaderVisibility=D3D12_SHADER_VISIBILITY_PIXEL;mixed[2].ParameterType=D3D12_ROOT_PARAMETER_TYPE_SRV;mixed[2].Descriptor={3,6};desc={3,mixed,0,nullptr,D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT};
    layouts.push_back(capture(device.Get(),desc,retained));
    device.Reset();ComPtr<ID3D12Device> parent;assert(retained->GetDevice(IID_PPV_ARGS(&parent))==S_OK&&parent->GetNodeCount()==1);retained.Reset();
    std::printf("{\"scenario\":\"root-signatures-v1\",\"layouts\":[");for(size_t row=0;row<layouts.size();++row){if(row)std::putchar(',');std::putchar('[');for(size_t i=0;i<layouts[row].size();++i)std::printf("%s%u",i?",":"",unsigned(layouts[row][i]));std::putchar(']');}std::puts("],\"versioned_identical\":true,\"metadata\":true,\"parent_lifetime\":true}");
}
