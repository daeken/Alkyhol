// Copyright (c) Alkyhol contributors. MIT licensed.
// Host-only Metal adapter discovery, not a Windows hardware comparison or renderer.
#include <Windows.h>
#include <dxgi1_6.h>
#include <d3d12.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <codecvt>
#include <locale>
#include <vector>
using Microsoft::WRL::ComPtr;
extern "C" HRESULT AlkyholCreateMetadataObject32(ID3D12Object**);
static_assert(sizeof(wchar_t)==4,"source ABI uses UTF-32 WCHAR");
static_assert(sizeof(DXGI_ADAPTER_DESC)==560&&sizeof(DXGI_ADAPTER_DESC1)==568,"source descriptor layouts");
void json_string(const std::string& value){std::putchar('"');for(unsigned char c:value){if(c=='"'||c=='\\'){std::putchar('\\');std::putchar(c);}else if(c<32)std::printf("\\u%04x",unsigned(c));else std::putchar(c);}std::putchar('"');}
void metadata(IDXGIObject* object){
    const GUID key={0x12480000,1,2,{3,4,5,6,7,8,9,10}};
    const unsigned char input[]={9,0,0xff,4};assert(object->SetPrivateData(key,sizeof(input),input)==S_OK);
    unsigned char output[8];std::memset(output,0xa5,sizeof(output));UINT size=3;
    assert(object->GetPrivateData(key,&size,output)==DXGI_ERROR_MORE_DATA&&size==4&&output[0]==0xa5);
    size=8;assert(object->GetPrivateData(key,&size,output)==S_OK&&size==4&&std::memcmp(input,output,4)==0&&output[4]==0xa5);
    ComPtr<ID3D12Object> foreign;assert(AlkyholCreateMetadataObject32(&foreign)==S_OK);
    assert(object->SetPrivateDataInterface(key,foreign.Get())==S_OK);IUnknown* copy=nullptr;size=sizeof(copy);
    assert(object->GetPrivateData(key,&size,&copy)==S_OK&&copy==foreign.Get());copy->Release();foreign.Reset();
    assert(object->SetPrivateData(key,0,nullptr)==S_OK);
}
int main(){
    void* failed=reinterpret_cast<void*>(1);GUID unknown={0xaaaaaaaa,0,0,{0}};
    assert(CreateDXGIFactory2(0,__uuidof(IDXGIFactory6),nullptr)==E_POINTER);
    assert(CreateDXGIFactory2(0,unknown,&failed)==E_NOINTERFACE&&!failed);
    failed=reinterpret_cast<void*>(1);assert(CreateDXGIFactory2(1,__uuidof(IDXGIFactory6),&failed)==E_NOTIMPL&&!failed);
    ComPtr<IDXGIFactory6> factory;assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
    const GUID ids[]={__uuidof(IUnknown),__uuidof(IDXGIObject),__uuidof(IDXGIFactory),__uuidof(IDXGIFactory1),__uuidof(IDXGIFactory2),__uuidof(IDXGIFactory3),__uuidof(IDXGIFactory4),__uuidof(IDXGIFactory5),__uuidof(IDXGIFactory6)};
    for(const GUID& id:ids){void* queried=nullptr;assert(factory->QueryInterface(id,&queried)==S_OK&&queried==factory.Get());static_cast<IUnknown*>(queried)->Release();}
    assert(factory->GetCreationFlags()==0&&factory->IsCurrent());
    assert(factory->GetParent(__uuidof(IUnknown),&failed)==E_NOINTERFACE&&!failed);metadata(factory.Get());
    std::vector<ComPtr<IDXGIAdapter1>> keep;
    std::printf("{\"scenario\":\"metal-dxgi-factory-v1\",\"adapters\":[");
    for(UINT index=0;;++index){
        ComPtr<IDXGIAdapter1> adapter;HRESULT hr=factory->EnumAdapters1(index,&adapter);
        if(hr==DXGI_ERROR_NOT_FOUND){assert(!adapter.Get());break;}assert(hr==S_OK&&adapter.Get());
        struct Guarded{uint64_t before;DXGI_ADAPTER_DESC1 desc;uint64_t after;} guarded{};guarded.before=0x1122334455667788;guarded.after=0x8877665544332211;
        assert(adapter->GetDesc1(&guarded.desc)==S_OK&&guarded.before==0x1122334455667788&&guarded.after==0x8877665544332211);
        auto& d=guarded.desc;assert(d.Description[0]&&d.Description[127]==0&&d.Flags==0);
        assert(d.VendorId==0&&d.DeviceId==0&&d.SubSysId==0&&d.Revision==0&&d.DedicatedSystemMemory==0&&d.SharedSystemMemory==0);
        DXGI_ADAPTER_DESC old{};assert(adapter->GetDesc(&old)==S_OK&&std::memcmp(old.Description,d.Description,sizeof(d.Description))==0&&old.DedicatedVideoMemory==d.DedicatedVideoMemory);
        assert(adapter->GetDesc(nullptr)==E_POINTER&&adapter->GetDesc1(nullptr)==E_POINTER);
        ComPtr<IUnknown> identity;assert(adapter.As(&identity)==S_OK&&identity.Get()==adapter.Get());
        ComPtr<IDXGIAdapter> base;assert(factory->EnumAdapters(index,&base)==S_OK);DXGI_ADAPTER_DESC base_desc{};assert(base->GetDesc(&base_desc)==S_OK&&base_desc.AdapterLuid.LowPart==d.AdapterLuid.LowPart);
        ComPtr<IDXGIAdapter1> roundtrip;assert(factory->EnumAdapterByLuid(d.AdapterLuid,IID_PPV_ARGS(&roundtrip))==S_OK);
        DXGI_ADAPTER_DESC1 roundtrip_desc{};assert(roundtrip->GetDesc1(&roundtrip_desc)==S_OK&&std::memcmp(&roundtrip_desc.AdapterLuid,&d.AdapterLuid,sizeof(LUID))==0);
        ComPtr<IDXGIAdapter1> preference;assert(factory->EnumAdapterByGpuPreference(index,DXGI_GPU_PREFERENCE_UNSPECIFIED,IID_PPV_ARGS(&preference))==S_OK);
        ComPtr<IDXGIFactory6> parent;assert(adapter->GetParent(IID_PPV_ARGS(&parent))==S_OK&&parent.Get()==factory.Get());
        failed=reinterpret_cast<void*>(1);assert(adapter->QueryInterface(__uuidof(ID3D12Device),&failed)==E_NOINTERFACE&&!failed);
        ComPtr<IDXGIOutput> output;assert(adapter->EnumOutputs(0,&output)==E_NOTIMPL&&!output.Get());
        LARGE_INTEGER version{};version.QuadPart=123;assert(adapter->CheckInterfaceSupport(__uuidof(ID3D12Device),&version)==E_NOINTERFACE&&version.QuadPart==0);
        metadata(adapter.Get());
        const uint64_t registry=uint64_t(d.AdapterLuid.LowPart)|(uint64_t(uint32_t(d.AdapterLuid.HighPart))<<32);
        if(index)std::putchar(',');std::printf("{\"name\":");json_string(std::wstring_convert<std::codecvt_utf8<wchar_t>>{}.to_bytes(d.Description));
        std::printf(",\"registry_id\":%llu,\"virtual_video_budget\":%llu}",static_cast<unsigned long long>(registry),static_cast<unsigned long long>(d.DedicatedVideoMemory));keep.push_back(adapter);
    }
    assert(!keep.empty());
    HWND window=reinterpret_cast<HWND>(1);assert(factory->MakeWindowAssociation(nullptr,0)==E_NOTIMPL&&factory->GetWindowAssociation(&window)==E_NOTIMPL&&!window);
    ComPtr<IDXGISwapChain> swap;assert(factory->CreateSwapChain(nullptr,nullptr,&swap)==E_NOTIMPL&&!swap.Get());
    ComPtr<IDXGIAdapter> software;assert(factory->CreateSoftwareAdapter(nullptr,&software)==E_NOTIMPL&&!software.Get());assert(!factory->IsWindowedStereoEnabled());
    ComPtr<IDXGISwapChain1> swap1;assert(factory->CreateSwapChainForHwnd(nullptr,nullptr,nullptr,nullptr,nullptr,&swap1)==E_NOTIMPL&&!swap1.Get());
    assert(factory->CreateSwapChainForCoreWindow(nullptr,nullptr,nullptr,nullptr,&swap1)==E_NOTIMPL&&!swap1.Get());
    assert(factory->CreateSwapChainForComposition(nullptr,nullptr,nullptr,&swap1)==E_NOTIMPL&&!swap1.Get());
    LUID shared={1,1};assert(factory->GetSharedResourceAdapterLuid(nullptr,&shared)==E_NOTIMPL&&!shared.LowPart&&!shared.HighPart);
    DWORD cookie=17;assert(factory->RegisterStereoStatusWindow(nullptr,0,&cookie)==E_NOTIMPL&&!cookie);cookie=17;assert(factory->RegisterStereoStatusEvent(nullptr,&cookie)==E_NOTIMPL&&!cookie);factory->UnregisterStereoStatus(0);
    cookie=17;assert(factory->RegisterOcclusionStatusWindow(nullptr,0,&cookie)==E_NOTIMPL&&!cookie);cookie=17;assert(factory->RegisterOcclusionStatusEvent(nullptr,&cookie)==E_NOTIMPL&&!cookie);factory->UnregisterOcclusionStatus(0);
    failed=reinterpret_cast<void*>(1);assert(factory->EnumWarpAdapter(__uuidof(IDXGIAdapter1),&failed)==E_NOTIMPL&&!failed);
    BOOL tearing=FALSE;assert(factory->CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING,&tearing,sizeof(tearing))==E_NOTIMPL);
    for(auto preference:{DXGI_GPU_PREFERENCE_MINIMUM_POWER,DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE}){failed=reinterpret_cast<void*>(1);assert(factory->EnumAdapterByGpuPreference(0,preference,__uuidof(IDXGIAdapter1),&failed)==E_NOTIMPL&&!failed);}
    failed=reinterpret_cast<void*>(1);assert(factory->EnumAdapterByLuid({0xffffffff,0x7fffffff},__uuidof(IDXGIAdapter1),&failed)==DXGI_ERROR_NOT_FOUND&&!failed);
    IUnknown* saved=factory.Get();factory.Reset();
    for(auto& adapter:keep){ComPtr<IUnknown> parent;assert(adapter->GetParent(IID_PPV_ARGS(&parent))==S_OK&&parent.Get()==saved);DXGI_ADAPTER_DESC1 desc{};assert(adapter->GetDesc1(&desc)==S_OK);}
    std::puts("],\"parent_lifetime\":true,\"unsupported_boundaries\":true,\"descriptor_bytes\":568}");
}
