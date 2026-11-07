// Copyright (c) Alkyhol contributors. MIT licensed.
// Links the original pinned CommandListManager/CommandAllocatorPool objects.
// This supplies only a caller and the original global; it does not rewrite them.
#include <Windows.h>
#include <d3d12.h>
#include <wrl/client.h>
#include "CommandListManager.h"
#include <cassert>
#include <cstdio>
extern "C" HRESULT AlkyholCreateMetalDeviceSubset(IUnknown*,REFIID,void**);
namespace Graphics { CommandListManager g_CommandManager; }
int main(){
    Microsoft::WRL::ComPtr<ID3D12Device> device;
    assert(AlkyholCreateMetalDeviceSubset(nullptr,IID_PPV_ARGS(&device))==S_OK);
    auto& manager=Graphics::g_CommandManager;
    manager.Create(device.Get());
    uint64_t final_values[3]={};unsigned index=0;
    for(auto type:{D3D12_COMMAND_LIST_TYPE_DIRECT,D3D12_COMMAND_LIST_TYPE_COMPUTE,D3D12_COMMAND_LIST_TYPE_COPY}){
        auto& queue=manager.GetQueue(type);assert(queue.IsReady());
        auto desc=queue.GetCommandQueue()->GetDesc();assert(desc.Type==type&&desc.NodeMask==1);
        for(unsigned i=1;i<=8;++i){auto fence=queue.IncrementFence();assert(fence==((uint64_t(type)<<56)|i));manager.WaitForFence(fence);assert(manager.IsFenceComplete(fence));}
        final_values[index++]=queue.GetNextFenceValue();queue.WaitForIdle();
    }
    manager.IdleGPU();manager.Shutdown();assert(!manager.GetGraphicsQueue().IsReady()&&!manager.GetComputeQueue().IsReady()&&!manager.GetCopyQueue().IsReady());
    std::printf("{\"scenario\":\"original-command-manager-v1\",\"next_values\":[%llu,%llu,%llu],\"queues\":3,\"increments_per_queue\":8,\"idle\":true,\"shutdown\":true}\n",static_cast<unsigned long long>(final_values[0]),static_cast<unsigned long long>(final_values[1]),static_cast<unsigned long long>(final_values[2]));
}
