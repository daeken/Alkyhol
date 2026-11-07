// Copyright (c) Alkyhol contributors. MIT licensed.
#include <d3dcompiler.h>
#include <roapi.h>
#include <wrl/client.h>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <thread>
#include <vector>
struct Apartment {
    Apartment(){const HRESULT result=RoInitialize(RO_INIT_MULTITHREADED);assert(SUCCEEDED(result));}
    ~Apartment(){RoUninitialize();}
};
int main() {
    Apartment apartment;
    std::fputs("COM probe: begin identity/ownership\n",stderr);
    using Microsoft::WRL::ComPtr;
    ComPtr<ID3DBlob> blob;
    assert(D3DCreateBlob(37,&blob)==S_OK);
    assert(blob->GetBufferSize()==37);
    std::memset(blob->GetBufferPointer(),0xa7,37);
    ComPtr<IUnknown> base;assert(SUCCEEDED(blob.As(&base)));
    ComPtr<ID3DBlob> again;assert(SUCCEEDED(base.As(&again)));
    assert(again.Get()==blob.Get());
    auto moved=std::move(again);assert(!again.Get() && moved.Get()==blob.Get());
    ComPtr<ID3DBlob> copied=moved;moved.Reset();
    assert(copied->GetBufferSize()==37);
    assert(static_cast<const unsigned char*>(copied->GetBufferPointer())[36]==0xa7);
    GUID unknown={0x11223344,0,0,{0}};
    void* failed=reinterpret_cast<void*>(1);
    assert(blob->QueryInterface(unknown,&failed)==E_NOINTERFACE && failed==nullptr);
    std::fputs("COM probe: identity/ownership passed; begin shared-MTA references\n",stderr);
    std::vector<std::thread> threads;
    for(int t=0;t<8;++t)threads.emplace_back([raw=blob.Get()]{Apartment thread_apartment;for(int i=0;i<2000;++i){ComPtr<ID3DBlob> local=raw;assert(local->GetBufferSize()==37);}});
    for(auto& t:threads)t.join();
    std::fputs("COM probe: shared-MTA references passed; begin final release\n",stderr);
    auto* detached=copied.Detach();assert(!copied.Get());copied.Attach(detached);
    blob.Reset();base.Reset();assert(copied->GetBufferSize()==37);copied.Reset();
    std::puts("COM blob: C++ identity, WRL ownership, buffer and concurrent refs passed");
}
