// Copyright (c) Alkyhol contributors. MIT licensed.
#include <d3d12.h>
#include <cstdio>
int wWinMain(HINSTANCE,HINSTANCE,LPWSTR,int);
int main(){
    // The original Release build ignores failed HRESULTs. Do not let an absent
    // COM device become an unexplained null-interface crash in original code.
    const HRESULT status=D3D12CreateDevice(nullptr,D3D_FEATURE_LEVEL_11_0,IID_ID3D12Device,nullptr);
    if(FAILED(status)){
        std::fprintf(stderr,"Original MiniEngine compiled and linked. Runtime unavailable: D3D12 COM device creation returned 0x%08x.\n",unsigned(status));
        return 78;
    }
    return wWinMain(GetModuleHandleW(nullptr),nullptr,nullptr,SW_SHOWDEFAULT);
}
