// Original MIT finite PE32 source facade. Native CI also checks the real SDK.
#pragma once
#define WINAPI __stdcall
#define NULL 0
#define TRUE 1
#define FALSE 0
#define SUCCEEDED(v) ((HRESULT)(v)>=0)
#define FAILED(v) ((HRESULT)(v)<0)
#define S_OK 0
#define E_FAIL ((HRESULT)0x80004005)
#define UNREFERENCED_PARAMETER(x) (void)(x)
typedef unsigned int UINT,DWORD,ULONG,WPARAM;typedef int INT,BOOL,HRESULT,LRESULT,LPARAM;typedef wchar_t WCHAR;typedef WCHAR* LPWSTR;typedef void VOID;typedef void* HWND;typedef void* HINSTANCE;
struct GUID{unsigned int a;unsigned short b,c;unsigned char d[8];};
inline void ZeroMemory(void* p,UINT n){unsigned char* b=(unsigned char*)p;for(UINT i=0;i<n;i++)b[i]=0;}
#define D3D_SDK_VERSION 32
#define D3DADAPTER_DEFAULT 0
#define D3DDEVTYPE_HAL 1
#define D3DCREATE_SOFTWARE_VERTEXPROCESSING 0x20
#define D3DSWAPEFFECT_DISCARD 1
#define D3DFMT_UNKNOWN 0
#define D3DFMT_X8R8G8B8 22
#define D3DCLEAR_TARGET 1
#define D3DCOLOR_XRGB(r,g,b) (0xff000000U|((DWORD)(r)<<16)|((DWORD)(g)<<8)|(DWORD)(b))
struct D3DPRESENT_PARAMETERS{UINT BackBufferWidth,BackBufferHeight,BackBufferFormat,BackBufferCount,MultiSampleType,MultiSampleQuality,SwapEffect;HWND hDeviceWindow;BOOL Windowed,EnableAutoDepthStencil;UINT AutoDepthStencilFormat,Flags,FullScreen_RefreshRateInHz,PresentationInterval;};
struct D3DSURFACE_DESC{UINT Format,Type,Usage,Pool,MultiSampleType,MultiSampleQuality,Width,Height;};
struct D3DLOCKED_RECT{INT Pitch;void* pBits;};

struct IDirect3D9; struct IDirect3DDevice9; struct IDirect3DSurface9;
struct IDirect3D9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual void __stdcall UnsupportedSlot3()=0;
 virtual UINT __stdcall GetAdapterCount()=0;
 virtual HRESULT __stdcall GetAdapterIdentifier(UINT,DWORD,void*)=0;
 virtual void __stdcall UnsupportedSlot6()=0;
 virtual void __stdcall UnsupportedSlot7()=0;
 virtual void __stdcall UnsupportedSlot8()=0;
 virtual void __stdcall UnsupportedSlot9()=0;
 virtual void __stdcall UnsupportedSlot10()=0;
 virtual void __stdcall UnsupportedSlot11()=0;
 virtual void __stdcall UnsupportedSlot12()=0;
 virtual void __stdcall UnsupportedSlot13()=0;
 virtual void __stdcall UnsupportedSlot14()=0;
 virtual void __stdcall UnsupportedSlot15()=0;
 virtual HRESULT __stdcall CreateDevice(UINT,UINT,HWND,DWORD,D3DPRESENT_PARAMETERS*,IDirect3DDevice9**)=0;
};
struct IDirect3DDevice9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual HRESULT __stdcall TestCooperativeLevel()=0;
 virtual void __stdcall UnsupportedSlot4()=0;
 virtual void __stdcall UnsupportedSlot5()=0;
 virtual HRESULT __stdcall GetDirect3D(IDirect3D9**)=0;
 virtual void __stdcall UnsupportedSlot7()=0;
 virtual void __stdcall UnsupportedSlot8()=0;
 virtual HRESULT __stdcall GetCreationParameters(void*)=0;
 virtual void __stdcall UnsupportedSlot10()=0;
 virtual void __stdcall UnsupportedSlot11()=0;
 virtual void __stdcall UnsupportedSlot12()=0;
 virtual void __stdcall UnsupportedSlot13()=0;
 virtual void __stdcall UnsupportedSlot14()=0;
 virtual void __stdcall UnsupportedSlot15()=0;
 virtual void __stdcall UnsupportedSlot16()=0;
 virtual HRESULT __stdcall Present(const void*,const void*,HWND,const void*)=0;
 virtual HRESULT __stdcall GetBackBuffer(UINT,UINT,UINT,IDirect3DSurface9**)=0;
 virtual void __stdcall UnsupportedSlot19()=0;
 virtual void __stdcall UnsupportedSlot20()=0;
 virtual void __stdcall UnsupportedSlot21()=0;
 virtual void __stdcall UnsupportedSlot22()=0;
 virtual void __stdcall UnsupportedSlot23()=0;
 virtual void __stdcall UnsupportedSlot24()=0;
 virtual void __stdcall UnsupportedSlot25()=0;
 virtual void __stdcall UnsupportedSlot26()=0;
 virtual void __stdcall UnsupportedSlot27()=0;
 virtual void __stdcall UnsupportedSlot28()=0;
 virtual void __stdcall UnsupportedSlot29()=0;
 virtual void __stdcall UnsupportedSlot30()=0;
 virtual void __stdcall UnsupportedSlot31()=0;
 virtual HRESULT __stdcall GetRenderTargetData(IDirect3DSurface9*,IDirect3DSurface9*)=0;
 virtual void __stdcall UnsupportedSlot33()=0;
 virtual void __stdcall UnsupportedSlot34()=0;
 virtual void __stdcall UnsupportedSlot35()=0;
 virtual HRESULT __stdcall CreateOffscreenPlainSurface(UINT,UINT,UINT,UINT,IDirect3DSurface9**,void*)=0;
 virtual void __stdcall UnsupportedSlot37()=0;
 virtual void __stdcall UnsupportedSlot38()=0;
 virtual void __stdcall UnsupportedSlot39()=0;
 virtual void __stdcall UnsupportedSlot40()=0;
 virtual HRESULT __stdcall BeginScene()=0;
 virtual HRESULT __stdcall EndScene()=0;
 virtual HRESULT __stdcall Clear(DWORD,const void*,DWORD,DWORD,float,DWORD)=0;
 virtual void __stdcall UnsupportedSlot44()=0;
 virtual void __stdcall UnsupportedSlot45()=0;
 virtual void __stdcall UnsupportedSlot46()=0;
 virtual void __stdcall UnsupportedSlot47()=0;
 virtual void __stdcall UnsupportedSlot48()=0;
 virtual void __stdcall UnsupportedSlot49()=0;
 virtual void __stdcall UnsupportedSlot50()=0;
 virtual void __stdcall UnsupportedSlot51()=0;
 virtual void __stdcall UnsupportedSlot52()=0;
 virtual void __stdcall UnsupportedSlot53()=0;
 virtual void __stdcall UnsupportedSlot54()=0;
 virtual void __stdcall UnsupportedSlot55()=0;
 virtual void __stdcall UnsupportedSlot56()=0;
 virtual void __stdcall UnsupportedSlot57()=0;
 virtual void __stdcall UnsupportedSlot58()=0;
 virtual void __stdcall UnsupportedSlot59()=0;
 virtual void __stdcall UnsupportedSlot60()=0;
 virtual void __stdcall UnsupportedSlot61()=0;
 virtual void __stdcall UnsupportedSlot62()=0;
 virtual void __stdcall UnsupportedSlot63()=0;
 virtual void __stdcall UnsupportedSlot64()=0;
 virtual void __stdcall UnsupportedSlot65()=0;
 virtual void __stdcall UnsupportedSlot66()=0;
 virtual void __stdcall UnsupportedSlot67()=0;
 virtual void __stdcall UnsupportedSlot68()=0;
 virtual void __stdcall UnsupportedSlot69()=0;
 virtual void __stdcall UnsupportedSlot70()=0;
 virtual void __stdcall UnsupportedSlot71()=0;
 virtual void __stdcall UnsupportedSlot72()=0;
 virtual void __stdcall UnsupportedSlot73()=0;
 virtual void __stdcall UnsupportedSlot74()=0;
 virtual void __stdcall UnsupportedSlot75()=0;
 virtual void __stdcall UnsupportedSlot76()=0;
 virtual void __stdcall UnsupportedSlot77()=0;
 virtual void __stdcall UnsupportedSlot78()=0;
 virtual void __stdcall UnsupportedSlot79()=0;
 virtual void __stdcall UnsupportedSlot80()=0;
 virtual void __stdcall UnsupportedSlot81()=0;
 virtual void __stdcall UnsupportedSlot82()=0;
 virtual void __stdcall UnsupportedSlot83()=0;
 virtual void __stdcall UnsupportedSlot84()=0;
 virtual void __stdcall UnsupportedSlot85()=0;
 virtual void __stdcall UnsupportedSlot86()=0;
 virtual void __stdcall UnsupportedSlot87()=0;
 virtual void __stdcall UnsupportedSlot88()=0;
 virtual void __stdcall UnsupportedSlot89()=0;
 virtual void __stdcall UnsupportedSlot90()=0;
 virtual void __stdcall UnsupportedSlot91()=0;
 virtual void __stdcall UnsupportedSlot92()=0;
 virtual void __stdcall UnsupportedSlot93()=0;
 virtual void __stdcall UnsupportedSlot94()=0;
 virtual void __stdcall UnsupportedSlot95()=0;
 virtual void __stdcall UnsupportedSlot96()=0;
 virtual void __stdcall UnsupportedSlot97()=0;
 virtual void __stdcall UnsupportedSlot98()=0;
 virtual void __stdcall UnsupportedSlot99()=0;
 virtual void __stdcall UnsupportedSlot100()=0;
 virtual void __stdcall UnsupportedSlot101()=0;
 virtual void __stdcall UnsupportedSlot102()=0;
 virtual void __stdcall UnsupportedSlot103()=0;
 virtual void __stdcall UnsupportedSlot104()=0;
 virtual void __stdcall UnsupportedSlot105()=0;
 virtual void __stdcall UnsupportedSlot106()=0;
 virtual void __stdcall UnsupportedSlot107()=0;
 virtual void __stdcall UnsupportedSlot108()=0;
 virtual void __stdcall UnsupportedSlot109()=0;
 virtual void __stdcall UnsupportedSlot110()=0;
 virtual void __stdcall UnsupportedSlot111()=0;
 virtual void __stdcall UnsupportedSlot112()=0;
 virtual void __stdcall UnsupportedSlot113()=0;
 virtual void __stdcall UnsupportedSlot114()=0;
 virtual void __stdcall UnsupportedSlot115()=0;
 virtual void __stdcall UnsupportedSlot116()=0;
 virtual void __stdcall UnsupportedSlot117()=0;
 virtual void __stdcall UnsupportedSlot118()=0;
};
struct IDirect3DSurface9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual HRESULT __stdcall GetDevice(IDirect3DDevice9**)=0;
 virtual void __stdcall UnsupportedSlot4()=0;
 virtual void __stdcall UnsupportedSlot5()=0;
 virtual void __stdcall UnsupportedSlot6()=0;
 virtual void __stdcall UnsupportedSlot7()=0;
 virtual void __stdcall UnsupportedSlot8()=0;
 virtual void __stdcall UnsupportedSlot9()=0;
 virtual UINT __stdcall GetType()=0;
 virtual void __stdcall UnsupportedSlot11()=0;
 virtual HRESULT __stdcall GetDesc(D3DSURFACE_DESC*)=0;
 virtual HRESULT __stdcall LockRect(D3DLOCKED_RECT*,const void*,DWORD)=0;
 virtual HRESULT __stdcall UnlockRect()=0;
 virtual void __stdcall UnsupportedSlot15()=0;
 virtual void __stdcall UnsupportedSlot16()=0;
};
typedef IDirect3D9* LPDIRECT3D9;typedef IDirect3DDevice9* LPDIRECT3DDEVICE9;
extern "C" __declspec(dllimport) IDirect3D9* __stdcall Direct3DCreate9(UINT);
#define CS_CLASSDC 0x40
#define WM_DESTROY 2
#define WM_PAINT 15
#define WS_OVERLAPPEDWINDOW 0xcf0000
#define SW_SHOWDEFAULT 10
struct WNDCLASSEX{UINT cbSize,style;LRESULT(__stdcall* lpfnWndProc)(HWND,UINT,WPARAM,LPARAM);INT cbClsExtra,cbWndExtra;HINSTANCE hInstance;void* hIcon;void* hCursor;void* hbrBackground;const WCHAR* lpszMenuName;const WCHAR* lpszClassName;void* hIconSm;};
struct MSG{HWND hwnd;UINT message;WPARAM wParam;LPARAM lParam;DWORD time;INT x,y;};
extern "C" {
__declspec(dllimport) HINSTANCE __stdcall GetModuleHandleW(const WCHAR*);
__declspec(dllimport) UINT __stdcall RegisterClassExW(const WNDCLASSEX*);
__declspec(dllimport) HWND __stdcall CreateWindowExW(DWORD,const WCHAR*,const WCHAR*,DWORD,INT,INT,INT,INT,HWND,void*,HINSTANCE,void*);
__declspec(dllimport) LRESULT __stdcall DefWindowProcW(HWND,UINT,WPARAM,LPARAM);
__declspec(dllimport) VOID __stdcall PostQuitMessage(INT);
__declspec(dllimport) BOOL __stdcall ValidateRect(HWND,const void*);
__declspec(dllimport) BOOL __stdcall ShowWindow(HWND,INT);
__declspec(dllimport) BOOL __stdcall UpdateWindow(HWND);
__declspec(dllimport) BOOL __stdcall GetMessageW(MSG*,HWND,UINT,UINT);
__declspec(dllimport) BOOL __stdcall TranslateMessage(const MSG*);
__declspec(dllimport) LRESULT __stdcall DispatchMessageW(const MSG*);
__declspec(dllimport) BOOL __stdcall UnregisterClassW(const WCHAR*,HINSTANCE);
__declspec(dllimport) BOOL __stdcall DestroyWindow(HWND);
}
#define GetModuleHandle GetModuleHandleW
#define RegisterClassEx RegisterClassExW
#define DefWindowProc DefWindowProcW
#define GetMessage GetMessageW
#define DispatchMessage DispatchMessageW
#define UnregisterClass UnregisterClassW
#define CreateWindow(c,t,s,x,y,w,h,p,m,i,a) CreateWindowExW(0,c,t,s,x,y,w,h,p,m,i,a)

#define D3DBACKBUFFER_TYPE_MONO 0
#define D3DPOOL_SYSTEMMEM 2
#define D3DLOCK_READONLY 0x10
