// Copyright (c) Alkyhol contributors. MIT licensed.
// Independent Windows SDK observation; no Alkyhol library is linked.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <d3dcommon.h>
#include <wrl/client.h>
#include <atomic>
#include <cstdint>
#include <cstdio>
#include <cstring>
using Microsoft::WRL::ComPtr;
static_assert(sizeof(void*)==8,"reference scenario is 64-bit");
namespace {
std::atomic<unsigned> refs{0},adds{0},releases{0},destroyed{0};
class TrackedUnknown final:public IUnknown {
    std::atomic<ULONG> count{1};
public:
    TrackedUnknown(){refs.store(1);}
    ~TrackedUnknown(){++destroyed;}
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid,void** out) override{
        if(!out)return E_POINTER;*out=nullptr;
        if(iid!=__uuidof(IUnknown))return E_NOINTERFACE;
        *out=static_cast<IUnknown*>(this);AddRef();return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override{++adds;ULONG n=++count;refs.store(unsigned(n));return n;}
    ULONG STDMETHODCALLTYPE Release() override{++releases;ULONG n=--count;refs.store(unsigned(n));if(!n)delete this;return n;}
};
const GUID BYTES={0xadc15001,0x21ef,0x4f43,{0x91,0x26,1,2,3,4,5,6}};
const GUID INTERFACE={0xadc15002,0x21ef,0x4f43,{0x91,0x26,1,2,3,4,5,6}};
bool first=true;
void emit(const char* label,HRESULT hr,UINT size=0,const void* data=nullptr,UINT length=0,bool measured_hr=true){
    std::printf("%s{\"case\":\"%s\",\"hr\":",first?"":",",label);first=false;
    if(measured_hr)std::printf("%u",unsigned(hr));else std::printf("null");
    std::printf(",\"size\":%u,\"data\":\"",unsigned(size));
    const auto* bytes=static_cast<const unsigned char*>(data);
    for(UINT i=0;i<length;++i)std::printf("%02x",unsigned(bytes[i]));
    std::printf("\",\"refs\":%u,\"adds\":%u,\"releases\":%u,\"destroyed\":%u}",refs.load(),adds.load(),releases.load(),destroyed.load());
}
void event(const char* label){emit(label,0,0,nullptr,0,false);}
void query(ID3D12Object* object,const char* label,REFGUID key){UINT size=0;HRESULT hr=object->GetPrivateData(key,&size,nullptr);emit(label,hr,size);}
void read(ID3D12Object* object,const char* label,REFGUID key,UINT capacity){
    unsigned char bytes[64];std::memset(bytes,0xa5,sizeof(bytes));UINT size=capacity;
    HRESULT hr=object->GetPrivateData(key,&size,bytes);emit(label,hr,size,bytes,UINT(sizeof(bytes)));
}
void observe(ID3D12Object* object){
    read(object,"bytes.missing",BYTES,16);
    unsigned char original[]={0,0xff,0x41,0x80};
    emit("bytes.set",object->SetPrivateData(BYTES,UINT(sizeof(original)),original));original[0]=99;
    query(object,"bytes.size",BYTES);read(object,"bytes.small",BYTES,3);read(object,"bytes.exact",BYTES,4);read(object,"bytes.large",BYTES,16);
    unsigned char replacement[]={0x73,0x00};emit("bytes.replace",object->SetPrivateData(BYTES,UINT(sizeof(replacement)),replacement));read(object,"bytes.replaced",BYTES,16);
    emit("bytes.remove",object->SetPrivateData(BYTES,0,nullptr));query(object,"bytes.removed",BYTES);
    emit("bytes.remove_missing",object->SetPrivateData(BYTES,0,nullptr));
    emit("bytes.zero_nonnull",object->SetPrivateData(BYTES,0,original));query(object,"bytes.zero_size",BYTES);read(object,"bytes.zero_read",BYTES,16);
    auto* tracked=new TrackedUnknown;
    emit("interface.set",object->SetPrivateDataInterface(INTERFACE,tracked));query(object,"interface.size",INTERFACE);read(object,"interface.small",INTERFACE,UINT(sizeof(void*)-1));
    IUnknown* retrieved=nullptr;UINT size=UINT(sizeof(retrieved));HRESULT hr=object->GetPrivateData(INTERFACE,&size,&retrieved);
    unsigned char same=retrieved==static_cast<IUnknown*>(tracked)?1:0;emit("interface.get",hr,size,&same,1);
    if(SUCCEEDED(hr)&&retrieved)retrieved->Release();event("interface.release_copy");
    emit("interface.replace_same",object->SetPrivateDataInterface(INTERFACE,tracked));
    emit("interface.replace_bytes",object->SetPrivateData(INTERFACE,UINT(sizeof(replacement)),replacement));read(object,"interface.bytes",INTERFACE,16);
    emit("interface.restore",object->SetPrivateDataInterface(INTERFACE,tracked));
    emit("interface.clear",object->SetPrivateDataInterface(INTERFACE,nullptr));query(object,"interface.cleared",INTERFACE);
    emit("interface.clear_missing",object->SetPrivateDataInterface(INTERFACE,nullptr));
    emit("interface.retained_until_destroy",object->SetPrivateDataInterface(INTERFACE,tracked));
    tracked->Release();event("interface.release_caller");
    emit("name.set",object->SetName(L"metadata \u03bb \U0001f338"));query(object,"name.size",WKPDID_D3DDebugObjectNameW);read(object,"name.read",WKPDID_D3DDebugObjectNameW,64);
    emit("name.empty",object->SetName(L""));query(object,"name.empty_size",WKPDID_D3DDebugObjectNameW);
}
bool checked(HRESULT hr,const char* call){if(SUCCEEDED(hr))return true;std::fprintf(stderr,"%s failed: %08x\n",call,unsigned(hr));return false;}
}
int main(){
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;ComPtr<ID3D12Fence> fence;ComPtr<ID3D12Object> object;
    if(!checked(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory)),"CreateDXGIFactory2")||!checked(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter)),"EnumWarpAdapter"))return 1;
    DXGI_ADAPTER_DESC1 desc{};if(!checked(adapter->GetDesc1(&desc),"GetDesc1")||!(desc.Flags&DXGI_ADAPTER_FLAG_SOFTWARE))return 2;
    std::fprintf(stderr,"adapter: D3D12 WARP vendor=%u device=%u flags=%u\n",unsigned(desc.VendorId),unsigned(desc.DeviceId),unsigned(desc.Flags));
    if(!checked(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device)),"D3D12CreateDevice")||!checked(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&fence)),"CreateFence")||!checked(fence.As(&object),"QueryInterface(ID3D12Object)"))return 3;
    std::printf("{\"scenario\":\"d3d12-object-private-data-v1\",\"records\":[");observe(object.Get());
    fence.Reset();object.Reset();device.Reset();event("object.destroyed");
    std::puts("]}");return 0;
}
