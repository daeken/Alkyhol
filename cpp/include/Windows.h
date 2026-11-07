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
#include "alkyhol_printf.h"
template<size_t N,class... A> inline int swprintf_s(wchar_t(&d)[N],const wchar_t* f,A...a){return alkyhol_swprintf(d,N,f,a...);}
#include "alkyhol_crt.h"
class ComputeContext;
#define CreateEvent CreateEventW
extern "C" HANDLE CreateEventW(const SECURITY_ATTRIBUTES*,BOOL,BOOL,LPCWSTR);
extern "C" BOOL SetEvent(HANDLE);
extern "C" DWORD WaitForSingleObject(HANDLE,DWORD);
extern "C" BOOL QueryPerformanceCounter(LARGE_INTEGER*);
extern "C" BOOL QueryPerformanceFrequency(LARGE_INTEGER*);
#define INFINITE 0xffffffffu
#define WAIT_OBJECT_0 0
#define WAIT_TIMEOUT 258
#define SWP_NOSIZE 0x0001
#define SWP_NOMOVE 0x0002
#define SWP_NOZORDER 0x0004
#define SWP_NOACTIVATE 0x0010
extern "C" BOOL SetWindowPos(HWND,HWND,int,int,int,int,UINT);
extern "C" BOOL GetClientRect(HWND,RECT*);
#define CP_ACP 0
#define CP_UTF8 65001
extern "C" int MultiByteToWideChar(UINT,DWORD,LPCSTR,int,LPWSTR,int);
extern "C" int WideCharToMultiByte(UINT,DWORD,LPCWSTR,int,char*,int,LPCSTR,BOOL*);
#define _Outptr_opt_
#define _Inout_updates_all_(...)
#define _Inout_updates_all_opt_(...)
#define MB_PRECOMPOSED 1
#define CALLBACK
#define __MINGW_EXTENSION
#define DECLSPEC_IMPORT
#define WINBASEAPI
#define LOBYTE(w) ((BYTE)((uintptr_t)(w)&0xff))
#define HIBYTE(w) ((BYTE)(((uintptr_t)(w)>>8)&0xff))
#define LOWORD(w) ((WORD)((uintptr_t)(w)&0xffff))
#define HIWORD(w) ((WORD)(((uintptr_t)(w)>>16)&0xffff))
typedef DWORD* LPDWORD;
typedef LONG* LPLONG;
typedef BYTE* LPBYTE;
typedef int16_t SHORT;
typedef uint16_t ATOM;
typedef ULONG_PTR WPARAM;
typedef LONG_PTR LPARAM,LRESULT;
typedef LRESULT(CALLBACK* WNDPROC)(HWND,UINT,WPARAM,LPARAM);
struct WNDCLASSEXW {UINT cbSize,style;WNDPROC lpfnWndProc;int cbClsExtra,cbWndExtra;HINSTANCE hInstance;HICON hIcon;HCURSOR hCursor;HBRUSH hbrBackground;LPCWSTR lpszMenuName,lpszClassName;HICON hIconSm;};
struct MSG {HWND hwnd;UINT message;WPARAM wParam;LPARAM lParam;DWORD time;POINT pt;DWORD lPrivate;};
#define WNDCLASSEX WNDCLASSEXW
#define CS_HREDRAW 2
#define CS_VREDRAW 1
#define COLOR_WINDOW 5
#define WS_OVERLAPPEDWINDOW 0xcf0000u
#define CW_USEDEFAULT ((int)0x80000000u)
#define PM_REMOVE 1
#define WM_SIZE 5
#define WM_DESTROY 2
#define WM_QUIT 0x12
#define SW_SHOWDEFAULT 10
#define MAKEINTRESOURCEW(i) ((LPWSTR)(uintptr_t)(WORD)(i))
#define IDI_APPLICATION MAKEINTRESOURCEW(32512)
#define IDC_ARROW MAKEINTRESOURCEW(32512)
extern "C" HICON LoadIconW(HINSTANCE,LPCWSTR);
extern "C" HCURSOR LoadCursorW(HINSTANCE,LPCWSTR);
extern "C" ATOM RegisterClassExW(const WNDCLASSEXW*);
extern "C" BOOL AdjustWindowRect(RECT*,DWORD,BOOL);
extern "C" HWND CreateWindowExW(DWORD,LPCWSTR,LPCWSTR,DWORD,int,int,int,int,HWND,HMENU,HINSTANCE,void*);
extern "C" BOOL ShowWindow(HWND,int);
extern "C" BOOL PeekMessageW(MSG*,HWND,UINT,UINT,UINT);
extern "C" BOOL TranslateMessage(const MSG*);
extern "C" LRESULT DispatchMessageW(const MSG*);
extern "C" LRESULT DefWindowProcW(HWND,UINT,WPARAM,LPARAM);
extern "C" void PostQuitMessage(int);
extern "C" DWORD GetLastError();
extern "C" void SetLastError(DWORD);
#define LoadIcon LoadIconW
#define LoadCursor LoadCursorW
#define RegisterClassEx RegisterClassExW
#define CreateWindow(a,b,c,d,e,f,g,h,i,j,k) CreateWindowExW(0,a,b,c,d,e,f,g,h,i,j,k)
#define PeekMessage PeekMessageW
#define DispatchMessage DispatchMessageW
#define DefWindowProc DefWindowProcW
#define ERROR_HANDLE_EOF 38
#define ERROR_INVALID_DATA 13
#define ERROR_NOT_SUPPORTED 50
#define ERROR_FILE_NOT_FOUND 2
#define ERROR_SUCCESS 0
inline constexpr HRESULT alkyhol_hresult_from_win32(int32_t code){return code<=0?code:int32_t(0x80070000u|(uint32_t(code)&0xffffu));}
#define HRESULT_FROM_WIN32(code) alkyhol_hresult_from_win32(static_cast<int32_t>(code))
struct FILETIME {DWORD dwLowDateTime,dwHighDateTime;};
typedef GUID* LPGUID;
extern "C" HMODULE GetModuleHandleW(LPCWSTR);
extern "C" HWND GetForegroundWindow();
extern "C" BOOL IsWindowVisible(HWND);
#define GetModuleHandle GetModuleHandleW
#include "malloc.h"
