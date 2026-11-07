// Copyright (c) Alkyhol contributors. MIT licensed.
#include "Windows.h"
#include <d3d12.h>
#include <dxgi1_6.h>
#include <dinput.h>
#include <xinput.h>
#include <roapi.h>
#include <shellapi.h>
#include <cassert>
#include <cstring>
#include <cstdio>
#include <string>
int main(){
    assert(RoInitialize(RO_INIT_MULTITHREADED)==S_OK);
    assert(RoInitialize(RO_INIT_MULTITHREADED)==S_FALSE);
    assert(FAILED(RoInitialize(RO_INIT_SINGLETHREADED)));RoUninitialize();RoUninitialize();
    const char utf8[]="A\xce\xbb\xf0\x9f\x8c\xb8";
    wchar_t wide[8]={};char bytes[32]={};
    assert(MultiByteToWideChar(CP_UTF8,0,utf8,-1,nullptr,0)==4);
    assert(MultiByteToWideChar(CP_UTF8,0,utf8,-1,wide,8)==4);
    assert(wide[0]==L'A'&&wide[1]==0x3bb&&wide[2]==0x1f338&&wide[3]==0);
    assert(WideCharToMultiByte(CP_UTF8,0,wide,-1,bytes,32,nullptr,nullptr)==sizeof(utf8));
    assert(std::memcmp(bytes,utf8,sizeof(utf8))==0);
    wchar_t guard[2]={91,92};assert(MultiByteToWideChar(CP_UTF8,0,utf8,-1,guard,2)==0&&GetLastError()==122&&guard[0]==91&&guard[1]==92);
    assert(MultiByteToWideChar(CP_ACP,0,"\xff",1,wide,8)==0&&GetLastError()==1113);
    assert(MultiByteToWideChar(CP_UTF8,8,"\xff",1,wide,8)==0&&GetLastError()==1113);
    assert(MultiByteToWideChar(CP_UTF8,0,"\xff",1,wide,8)==1&&wide[0]==0xfffd);
    wchar_t invalid[]={0xd800,0};assert(WideCharToMultiByte(CP_UTF8,0,invalid,-1,bytes,32,nullptr,nullptr)==0&&GetLastError()==1113);
    assert(MultiByteToWideChar(CP_UTF8,0,utf8,0,wide,8)==0&&GetLastError()==87);
    assert(WideCharToMultiByte(999,0,L"x",1,bytes,32,nullptr,nullptr)==0&&GetLastError()==50);
    HMODULE module=GetModuleHandleW(nullptr);assert(module);
    wchar_t executable[4096]={};DWORD length=GetModuleFileNameW(module,executable,4096);assert(length>1&&executable[length]==0);
    wchar_t shortname[2]={};assert(GetModuleFileNameW(nullptr,shortname,2)==2&&shortname[1]==0&&GetLastError()==122);
    assert(GetModuleHandleW(L"unsupported.dll")==nullptr&&GetLastError()==50);
    const wchar_t* path=L".\\alkyhol_\u03bb_runtime.tmp";
    FILE* file=nullptr;assert(fopen_s(&file,alkyhol_path(path).c_str(),"wb")==0);const char payload[]={0,1,2,3,4};assert(std::fwrite(payload,1,sizeof(payload),file)==sizeof(payload));std::fclose(file);
    CREATEFILE2_EXTENDED_PARAMETERS parameters={sizeof(parameters),FILE_ATTRIBUTE_NORMAL,FILE_FLAG_SEQUENTIAL_SCAN,0,nullptr,nullptr};
    assert(CreateFile2(path,GENERIC_READ,FILE_SHARE_READ,OPEN_EXISTING,&parameters)==INVALID_HANDLE_VALUE&&GetLastError()==50);
    HANDLE h=CreateFile2(path,GENERIC_READ,7,OPEN_EXISTING,&parameters);assert(h!=INVALID_HANDLE_VALUE);
    FILE_STANDARD_INFO info{};assert(sizeof(info)==24);
    assert(GetFileInformationByHandleEx(h,FileStandardInfo,&info,sizeof(info)));
    assert(info.EndOfFile.QuadPart==sizeof(payload)&&info.AllocationSize.QuadPart>=sizeof(payload)&&info.NumberOfLinks==1&&!info.Directory&&!info.DeletePending);
    assert(!GetFileInformationByHandleEx(h,FileBasicInfo,&info,sizeof(info))&&GetLastError()==50);
    assert(!GetFileInformationByHandleEx(h,FileStandardInfo,&info,1)&&GetLastError()==87);
    char data[16]={};DWORD read=0;assert(ReadFile(h,data,sizeof(data),&read,nullptr)&&read==sizeof(payload)&&std::memcmp(payload,data,read)==0);
    assert(CloseHandle(h));assert(!GetFileInformationByHandleEx(h,FileStandardInfo,&info,sizeof(info))&&GetLastError()==6);
    parameters.dwFileFlags=0x40000000;assert(CreateFile2(path,GENERIC_READ,1,3,&parameters)==INVALID_HANDLE_VALUE&&GetLastError()==50);
    assert(std::remove(alkyhol_path(path).c_str())==0);
    OutputDebugStringA("debug narrow\n");OutputDebugStringW(L"debug wide \u03bb\n");
    void* object=reinterpret_cast<void*>(1);
    assert(CreateDXGIFactory2(0,__uuidof(IDXGIFactory6),&object)==E_NOTIMPL&&!object);
    object=reinterpret_cast<void*>(1);assert(D3D12CreateDevice(nullptr,D3D_FEATURE_LEVEL_11_0,IID_ID3D12Device,&object)==E_NOTIMPL&&!object);
    assert(D3D12CreateDevice(nullptr,D3D_FEATURE_LEVEL_11_0,IID_ID3D12Device,nullptr)==E_NOTIMPL);
    object=reinterpret_cast<void*>(1);assert(D3D12GetDebugInterface(IID_ID3D12Device,&object)==E_NOTIMPL&&!object);
    assert(D3D12EnableExperimentalFeatures(0,nullptr,nullptr,nullptr)==E_NOTIMPL);
    ID3DBlob* blob=reinterpret_cast<ID3DBlob*>(1);ID3DBlob* error=reinterpret_cast<ID3DBlob*>(1);
    assert(D3D12SerializeRootSignature(nullptr,D3D_ROOT_SIGNATURE_VERSION_1,&blob,&error)==E_POINTER&&!blob&&error);
    assert(error->GetBufferSize()>1&&static_cast<const char*>(error->GetBufferPointer())[error->GetBufferSize()-1]==0);assert(error->Release()==0);
    blob=reinterpret_cast<ID3DBlob*>(1);error=reinterpret_cast<ID3DBlob*>(1);
    assert(D3D12SerializeVersionedRootSignature(nullptr,&blob,&error)==E_POINTER&&!blob&&error);assert(error->Release()==0);
    D3D12_ROOT_SIGNATURE_DESC root{};
    assert(D3D12SerializeRootSignature(&root,D3D_ROOT_SIGNATURE_VERSION_1,&blob,&error)==S_OK&&blob&&!error);
    assert(blob->GetBufferSize()==68&&std::memcmp(blob->GetBufferPointer(),"DXBC",4)==0);assert(blob->Release()==0);
    D3D12_VERSIONED_ROOT_SIGNATURE_DESC versioned{};versioned.Version=D3D_ROOT_SIGNATURE_VERSION_1_0;versioned.Desc_1_0=root;
    assert(D3D12SerializeVersionedRootSignature(&versioned,&blob,&error)==S_OK&&blob&&!error);assert(blob->Release()==0);
    versioned.Version=D3D_ROOT_SIGNATURE_VERSION_1_1;
    assert(D3D12SerializeVersionedRootSignature(&versioned,&blob,&error)==E_NOTIMPL&&!blob&&error);assert(error->Release()==0);
    object=reinterpret_cast<void*>(1);assert(DirectInput8Create(nullptr,0x800,IID_IDirectInput8W,&object,nullptr)==E_NOTIMPL&&!object);
    XINPUT_STATE state{};assert(XInputGetState(0,&state)==50);
    int count=19;assert(CommandLineToArgvW(L"program.exe",&count)==nullptr&&count==0&&GetLastError()==50);
    assert(GetCommandLineW()==nullptr&&GetLastError()==50);
    assert(!RegisterClassExW(nullptr)&&GetLastError()==50);
    assert(!CreateWindowExW(0,nullptr,nullptr,0,0,0,800,600,nullptr,nullptr,module,nullptr)&&GetLastError()==50);
    RECT rect={0,0,800,600};assert(!AdjustWindowRect(&rect,0,0)&&GetLastError()==50);
    assert(!LoadIconW(nullptr,IDI_APPLICATION)&&!LoadCursorW(nullptr,IDC_ARROW));
    assert(!GetForegroundWindow()&&!IsWindowVisible(nullptr)&&!ShowWindow(nullptr,1));
    assert(!SetWindowPos(nullptr,nullptr,0,0,800,600,0)&&!GetClientRect(nullptr,&rect));
    MSG message{};assert(!PeekMessageW(&message,nullptr,0,0,0)&&!TranslateMessage(&message));
    assert(DispatchMessageW(&message)==0&&GetLastError()==50);assert(DefWindowProcW(nullptr,0,0,0)==0&&GetLastError()==50);PostQuitMessage(0);assert(GetLastError()==50);
    std::puts("C++/Rust ABI: UTF-32 text, actual file I/O/metadata, COM nesting and honest unsupported boundaries passed");
}
