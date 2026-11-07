#pragma once
#include "basetsd.h"
#include "unknwn.h"
#include <cstring>
#include <cwchar>
#include <cstdarg>
#include <cstdio>
#ifndef __forceinline
#define __forceinline inline __attribute__((always_inline))
#endif
#define ZeroMemory(p,n) memset(p,0,n)
#define CopyMemory(d,s,n) memcpy(d,s,n)
extern "C" void OutputDebugStringA(const char*);
extern "C" void OutputDebugStringW(const wchar_t*);
#define OutputDebugString OutputDebugStringW
inline int vsprintf_s(char* d,size_t n,const char* fmt,va_list a){return vsnprintf(d,n,fmt,a);}
typedef void *HMODULE, *HMONITOR, *HINSTANCE, *HCURSOR, *HICON, *HBRUSH, *HMENU;

typedef unsigned char byte;
#define INVALID_HANDLE_VALUE ((HANDLE)(intptr_t)-1)
#define GENERIC_READ 0x80000000u
#define GENERIC_WRITE 0x40000000u
#define FILE_SHARE_READ 1
#define FILE_SHARE_WRITE 2
#define OPEN_EXISTING 3
#define FILE_ATTRIBUTE_NORMAL 0x80
#define FILE_FLAG_SEQUENTIAL_SCAN 0x08000000
#define SECURITY_ANONYMOUS 0
struct CREATEFILE2_EXTENDED_PARAMETERS { DWORD dwSize,dwFileAttributes,dwFileFlags,dwSecurityQosFlags;SECURITY_ATTRIBUTES* lpSecurityAttributes;HANDLE hTemplateFile; };
struct FILE_STANDARD_INFO { LARGE_INTEGER AllocationSize,EndOfFile;DWORD NumberOfLinks;BOOLEAN DeletePending,Directory; };
enum FILE_INFO_BY_HANDLE_CLASS {FileBasicInfo,FileStandardInfo};
extern "C" DWORD GetModuleFileNameW(HMODULE,wchar_t*,DWORD);
#define GetModuleFileName GetModuleFileNameW
extern "C" HANDLE CreateFile2(LPCWSTR,DWORD,DWORD,DWORD,const CREATEFILE2_EXTENDED_PARAMETERS*);
extern "C" BOOL GetFileInformationByHandleEx(HANDLE,FILE_INFO_BY_HANDLE_CLASS,void*,DWORD);
extern "C" BOOL ReadFile(HANDLE,void*,DWORD,DWORD*,void*);
extern "C" BOOL CloseHandle(HANDLE);
template<size_t N,class... A> inline int sprintf_s(char(&d)[N],const char* f,A...a){return snprintf(d,N,f,a...);}
#define ARRAYSIZE(x) (sizeof(x)/sizeof(*(x)))
template<size_t N,class... A> inline int swprintf_s(wchar_t(&d)[N],const wchar_t* f,A...a){return swprintf(d,N,f,a...);}
