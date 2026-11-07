// Copyright (c) Alkyhol contributors. MIT licensed.
// Finite synchronization behavior: native WARP vs the explicit Metal subset.
#include <Windows.h>
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <atomic>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <thread>
#include <vector>
using Microsoft::WRL::ComPtr;
#ifndef _WIN32
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
#endif
struct Event{HANDLE h=CreateEventW(nullptr,FALSE,FALSE,nullptr);Event(){assert(h);}~Event(){assert(CloseHandle(h));}};
int main(int argc,char** argv){
    (void)argc;(void)argv;
    ComPtr<IDXGIFactory4> factory;ComPtr<IDXGIAdapter1> adapter;ComPtr<ID3D12Device> device;
    assert(CreateDXGIFactory2(0,IID_PPV_ARGS(&factory))==S_OK);
#ifdef _WIN32
    assert(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter))==S_OK);assert(D3D12CreateDevice(adapter.Get(),D3D_FEATURE_LEVEL_11_0,IID_PPV_ARGS(&device))==S_OK);
#else
    assert(factory->EnumAdapters1(0,&adapter)==S_OK);assert(AlkyholCreateMetalDeviceSubset(adapter.Get(),IID_PPV_ARGS(&device))==S_OK);
#endif
    ComPtr<IUnknown> identity;assert(device.As(&identity)==S_OK);IUnknown* canonical=identity.Get();identity.Reset();factory.Reset();adapter.Reset();
    ComPtr<ID3D12Fence> fence;assert(device->CreateFence(5,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&fence))==S_OK);assert(fence->SetName(L"sync fence")==S_OK);
    std::vector<uint64_t> values;values.push_back(fence->GetCompletedValue());assert(values.back()==5);
    Event a,b;std::vector<unsigned> waits;
    assert(fence->SetEventOnCompletion(4,a.h)==S_OK);waits.push_back(WaitForSingleObject(a.h,0));assert(waits.back()==WAIT_OBJECT_0);
    assert(fence->SetEventOnCompletion(8,a.h)==S_OK);waits.push_back(WaitForSingleObject(a.h,0));assert(waits.back()==WAIT_TIMEOUT);
    assert(fence->Signal(7)==S_OK);values.push_back(fence->GetCompletedValue());waits.push_back(WaitForSingleObject(a.h,0));assert(waits.back()==WAIT_TIMEOUT);
    assert(fence->Signal(8)==S_OK);waits.push_back(WaitForSingleObject(a.h,5000));assert(waits.back()==WAIT_OBJECT_0);values.push_back(fence->GetCompletedValue());
    assert(fence->Signal(3)==S_OK);values.push_back(fence->GetCompletedValue());assert(values.back()==3);
    assert(fence->SetEventOnCompletion(6,a.h)==S_OK&&fence->SetEventOnCompletion(9,b.h)==S_OK);assert(fence->Signal(6)==S_OK);
    waits.push_back(WaitForSingleObject(a.h,5000));assert(waits.back()==WAIT_OBJECT_0);waits.push_back(WaitForSingleObject(b.h,0));assert(waits.back()==WAIT_TIMEOUT);
    assert(fence->Signal(9)==S_OK);waits.push_back(WaitForSingleObject(b.h,5000));assert(waits.back()==WAIT_OBJECT_0);values.push_back(fence->GetCompletedValue());
    assert(fence->SetEventOnCompletion(0,nullptr)==S_OK);
    std::atomic<bool> ready{false};std::thread blocking([&]{ready.store(true);assert(fence->SetEventOnCompletion(12,nullptr)==S_OK);});while(!ready.load())std::this_thread::yield();assert(fence->Signal(12)==S_OK);blocking.join();values.push_back(fence->GetCompletedValue());
    D3D12_COMMAND_QUEUE_DESC requested={D3D12_COMMAND_LIST_TYPE_COPY,0,D3D12_COMMAND_QUEUE_FLAG_NONE,0};ComPtr<ID3D12CommandQueue> queue;
    assert(device->CreateCommandQueue(&requested,IID_PPV_ARGS(&queue))==S_OK);assert(queue->SetName(L"sync queue")==S_OK);const auto desc=queue->GetDesc();
#ifndef _WIN32
    if(argc==2){
        if(!std::strcmp(argv[1],"trap-execute"))queue->ExecuteCommandLists(0,nullptr);
        if(!std::strcmp(argv[1],"trap-marker"))queue->SetMarker(0,nullptr,0);
        if(!std::strcmp(argv[1],"trap-begin"))queue->BeginEvent(0,nullptr,0);
        if(!std::strcmp(argv[1],"trap-end"))queue->EndEvent();
        if(!std::strcmp(argv[1],"trap-tiles"))queue->UpdateTileMappings(nullptr,0,nullptr,nullptr,nullptr,0,nullptr,nullptr,nullptr,D3D12_TILE_MAPPING_FLAG_NONE);
        if(!std::strcmp(argv[1],"trap-copy-tiles"))queue->CopyTileMappings(nullptr,nullptr,nullptr,nullptr,nullptr,D3D12_TILE_MAPPING_FLAG_NONE);
        return 99;
    }
#endif

    assert(fence->SetEventOnCompletion(21,a.h)==S_OK);assert(queue->Signal(fence.Get(),21)==S_OK);waits.push_back(WaitForSingleObject(a.h,5000));assert(waits.back()==WAIT_OBJECT_0);values.push_back(fence->GetCompletedValue());
    ComPtr<ID3D12Fence> checkpoint;assert(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&checkpoint))==S_OK);
    assert(checkpoint->SetEventOnCompletion(1,b.h)==S_OK);assert(queue->Signal(fence.Get(),10)==S_OK&&queue->Signal(fence.Get(),2)==S_OK&&queue->Signal(checkpoint.Get(),1)==S_OK);
    waits.push_back(WaitForSingleObject(b.h,5000));assert(waits.back()==WAIT_OBJECT_0);values.push_back(fence->GetCompletedValue());assert(values.back()==2);
    // Submission must retain state even when the caller releases the fence shell.
    ComPtr<ID3D12Fence> temporary;assert(device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&temporary))==S_OK);assert(temporary->SetEventOnCompletion(99,a.h)==S_OK);assert(queue->Signal(temporary.Get(),99)==S_OK);temporary.Reset();waits.push_back(WaitForSingleObject(a.h,5000));assert(waits.back()==WAIT_OBJECT_0);
#ifndef _WIN32
    assert(queue->Wait(fence.Get(),3)==E_NOTIMPL);uint64_t frequency=17,gpu=17,cpu=17;assert(queue->GetTimestampFrequency(&frequency)==E_NOTIMPL&&frequency==0);assert(queue->GetClockCalibration(&gpu,&cpu)==E_NOTIMPL&&!gpu&&!cpu);
#endif
    device.Reset();ComPtr<ID3D12Device> parent;assert(queue->GetDevice(IID_PPV_ARGS(&parent))==S_OK);ComPtr<IUnknown> parent_identity;assert(parent.As(&parent_identity)==S_OK&&parent_identity.Get()==canonical);
    ComPtr<ID3D12Device> fence_parent;assert(fence->GetDevice(IID_PPV_ARGS(&fence_parent))==S_OK);ComPtr<IUnknown> fence_identity;assert(fence_parent.As(&fence_identity)==S_OK&&fence_identity.Get()==canonical);
    std::printf("{\"scenario\":\"queue-fence-v1\",\"desc\":[%u,%d,%u,%u],\"values\":[",unsigned(desc.Type),int(desc.Priority),unsigned(desc.Flags),unsigned(desc.NodeMask));
    for(size_t i=0;i<values.size();++i)std::printf("%s%llu",i?",":"",static_cast<unsigned long long>(values[i]));std::printf("],\"waits\":[");for(size_t i=0;i<waits.size();++i)std::printf("%s%u",i?",":"",waits[i]);std::puts("],\"retained_submission\":true,\"parent_lifetime\":true}");
}
