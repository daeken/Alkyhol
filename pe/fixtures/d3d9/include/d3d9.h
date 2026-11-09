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
struct D3DDISPLAYMODE{UINT Width,Height,RefreshRate,Format;};
struct D3DVERTEXBUFFER_DESC{UINT Format,Type,Usage,Pool,Size,FVF;};
struct D3DGAMMARAMP{unsigned short red[256],green[256],blue[256];};
struct D3DLOCKED_RECT{INT Pitch;void* pBits;};

struct IDirect3DSwapChain9;
struct IDirect3DIndexBuffer9;
struct IDirect3DBaseTexture9;struct IDirect3DTexture9;
struct IDirect3DVertexShader9;struct IDirect3DPixelShader9;struct IDirect3DVertexDeclaration9;
struct D3DVERTEXELEMENT9{unsigned short Stream,Offset;unsigned char Type,Method,Usage,UsageIndex;};
struct IDirect3D9; struct IDirect3DDevice9; struct IDirect3DSurface9; struct IDirect3DVertexBuffer9;
struct IDirect3D9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual void __stdcall UnsupportedSlot3()=0;
 virtual UINT __stdcall GetAdapterCount()=0;
 virtual HRESULT __stdcall GetAdapterIdentifier(UINT,DWORD,void*)=0;
 virtual UINT __stdcall GetAdapterModeCount(UINT,UINT)=0;
 virtual HRESULT __stdcall EnumAdapterModes(UINT,UINT,UINT,D3DDISPLAYMODE*)=0;
 virtual HRESULT __stdcall GetAdapterDisplayMode(UINT,D3DDISPLAYMODE*)=0;
 virtual HRESULT __stdcall CheckDeviceType(UINT,UINT,UINT,UINT,BOOL)=0;
 virtual HRESULT __stdcall CheckDeviceFormat(UINT,UINT,UINT,DWORD,UINT,UINT)=0;
 virtual HRESULT __stdcall CheckDeviceMultiSampleType(UINT,UINT,UINT,BOOL,UINT,DWORD*)=0;
 virtual void __stdcall UnsupportedSlot12()=0;
 virtual void __stdcall UnsupportedSlot13()=0;
 virtual HRESULT __stdcall GetDeviceCaps(UINT,UINT,void*)=0;
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
 virtual HRESULT __stdcall GetSwapChain(UINT,IDirect3DSwapChain9**)=0;
 virtual UINT __stdcall GetNumberOfSwapChains()=0;
 virtual void __stdcall UnsupportedSlot16()=0;
 virtual HRESULT __stdcall Present(const void*,const void*,HWND,const void*)=0;
 virtual HRESULT __stdcall GetBackBuffer(UINT,UINT,UINT,IDirect3DSurface9**)=0;
 virtual void __stdcall UnsupportedSlot19()=0;
 virtual void __stdcall UnsupportedSlot20()=0;
 virtual void __stdcall SetGammaRamp(UINT,DWORD,const D3DGAMMARAMP*)=0;
 virtual void __stdcall UnsupportedSlot22()=0;
 virtual HRESULT __stdcall CreateTexture(UINT,UINT,UINT,DWORD,UINT,UINT,IDirect3DTexture9**,void*)=0;
 virtual void __stdcall UnsupportedSlot24()=0;
 virtual void __stdcall UnsupportedSlot25()=0;
 virtual HRESULT __stdcall CreateVertexBuffer(UINT,DWORD,DWORD,UINT,IDirect3DVertexBuffer9**,void*)=0;
 virtual HRESULT __stdcall CreateIndexBuffer(UINT,DWORD,UINT,UINT,IDirect3DIndexBuffer9**,void*)=0;
 virtual void __stdcall UnsupportedSlot28()=0;
 virtual HRESULT __stdcall CreateDepthStencilSurface(UINT,UINT,UINT,UINT,DWORD,BOOL,IDirect3DSurface9**,void*)=0;
 virtual void __stdcall UnsupportedSlot30()=0;
 virtual void __stdcall UnsupportedSlot31()=0;
 virtual HRESULT __stdcall GetRenderTargetData(IDirect3DSurface9*,IDirect3DSurface9*)=0;
 virtual void __stdcall UnsupportedSlot33()=0;
 virtual HRESULT __stdcall StretchRect(IDirect3DSurface9*,const void*,IDirect3DSurface9*,const void*,UINT)=0;
 virtual void __stdcall UnsupportedSlot35()=0;
 virtual HRESULT __stdcall CreateOffscreenPlainSurface(UINT,UINT,UINT,UINT,IDirect3DSurface9**,void*)=0;
 virtual HRESULT __stdcall SetRenderTarget(DWORD,IDirect3DSurface9*)=0;
 virtual HRESULT __stdcall GetRenderTarget(DWORD,IDirect3DSurface9**)=0;
 virtual HRESULT __stdcall SetDepthStencilSurface(IDirect3DSurface9*)=0;
 virtual HRESULT __stdcall GetDepthStencilSurface(IDirect3DSurface9**)=0;
 virtual HRESULT __stdcall BeginScene()=0;
 virtual HRESULT __stdcall EndScene()=0;
 virtual HRESULT __stdcall Clear(DWORD,const void*,DWORD,DWORD,float,DWORD)=0;
 virtual HRESULT __stdcall SetTransform(UINT,const float*)=0;
 virtual HRESULT __stdcall GetTransform(UINT,float*)=0;
 virtual HRESULT __stdcall MultiplyTransform(UINT,const float*)=0;
 virtual HRESULT __stdcall SetViewport(const void*)=0;
 virtual HRESULT __stdcall GetViewport(void*)=0;
 virtual void __stdcall UnsupportedSlot49()=0;
 virtual void __stdcall UnsupportedSlot50()=0;
 virtual void __stdcall UnsupportedSlot51()=0;
 virtual void __stdcall UnsupportedSlot52()=0;
 virtual void __stdcall UnsupportedSlot53()=0;
 virtual void __stdcall UnsupportedSlot54()=0;
 virtual void __stdcall UnsupportedSlot55()=0;
 virtual void __stdcall UnsupportedSlot56()=0;
 virtual HRESULT __stdcall SetRenderState(UINT,DWORD)=0;
 virtual HRESULT __stdcall GetRenderState(UINT,DWORD*)=0;
 virtual void __stdcall UnsupportedSlot59()=0;
 virtual void __stdcall UnsupportedSlot60()=0;
 virtual void __stdcall UnsupportedSlot61()=0;
 virtual void __stdcall UnsupportedSlot62()=0;
 virtual void __stdcall UnsupportedSlot63()=0;
 virtual HRESULT __stdcall GetTexture(DWORD,IDirect3DBaseTexture9**)=0;
 virtual HRESULT __stdcall SetTexture(DWORD,IDirect3DBaseTexture9*)=0;
 virtual HRESULT __stdcall GetTextureStageState(DWORD,UINT,DWORD*)=0;
 virtual HRESULT __stdcall SetTextureStageState(DWORD,UINT,DWORD)=0;
 virtual HRESULT __stdcall GetSamplerState(DWORD,UINT,DWORD*)=0;
 virtual HRESULT __stdcall SetSamplerState(DWORD,UINT,DWORD)=0;
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
 virtual HRESULT __stdcall DrawPrimitive(UINT,UINT,UINT)=0;
 virtual HRESULT __stdcall DrawIndexedPrimitive(UINT,INT,UINT,UINT,UINT,UINT)=0;
 virtual void __stdcall UnsupportedSlot83()=0;
 virtual void __stdcall UnsupportedSlot84()=0;
 virtual void __stdcall UnsupportedSlot85()=0;
 virtual HRESULT __stdcall CreateVertexDeclaration(const D3DVERTEXELEMENT9*,IDirect3DVertexDeclaration9**)=0;
 virtual HRESULT __stdcall SetVertexDeclaration(IDirect3DVertexDeclaration9*)=0;
 virtual HRESULT __stdcall GetVertexDeclaration(IDirect3DVertexDeclaration9**)=0;
 virtual HRESULT __stdcall SetFVF(DWORD)=0;
 virtual void __stdcall UnsupportedSlot90()=0;
 virtual HRESULT __stdcall CreateVertexShader(const DWORD*,IDirect3DVertexShader9**)=0;
 virtual HRESULT __stdcall SetVertexShader(IDirect3DVertexShader9*)=0;
 virtual HRESULT __stdcall GetVertexShader(IDirect3DVertexShader9**)=0;
 virtual HRESULT __stdcall SetVertexShaderConstantF(UINT,const float*,UINT)=0;
 virtual HRESULT __stdcall GetVertexShaderConstantF(UINT,float*,UINT)=0;
 virtual void __stdcall UnsupportedSlot96()=0;
 virtual void __stdcall UnsupportedSlot97()=0;
 virtual void __stdcall UnsupportedSlot98()=0;
 virtual void __stdcall UnsupportedSlot99()=0;
 virtual HRESULT __stdcall SetStreamSource(UINT,IDirect3DVertexBuffer9*,UINT,UINT)=0;
 virtual HRESULT __stdcall GetStreamSource(UINT,IDirect3DVertexBuffer9**,UINT*,UINT*)=0;
 virtual HRESULT __stdcall SetStreamSourceFreq(UINT,UINT)=0;
 virtual HRESULT __stdcall GetStreamSourceFreq(UINT,UINT*)=0;
 virtual HRESULT __stdcall SetIndices(IDirect3DIndexBuffer9*)=0;
 virtual HRESULT __stdcall GetIndices(IDirect3DIndexBuffer9**)=0;
 virtual HRESULT __stdcall CreatePixelShader(const DWORD*,IDirect3DPixelShader9**)=0;
 virtual HRESULT __stdcall SetPixelShader(IDirect3DPixelShader9*)=0;
 virtual HRESULT __stdcall GetPixelShader(IDirect3DPixelShader9**)=0;
 virtual HRESULT __stdcall SetPixelShaderConstantF(UINT,const float*,UINT)=0;
 virtual HRESULT __stdcall GetPixelShaderConstantF(UINT,float*,UINT)=0;
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
 virtual HRESULT __stdcall GetContainer(const GUID&,void**)=0;
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
__declspec(dllimport) BOOL __stdcall GetWindowInfo(HWND,void*);
__declspec(dllimport) BOOL __stdcall GetWindowRect(HWND,void*);
__declspec(dllimport) BOOL __stdcall GetClientRect(HWND,void*);
__declspec(dllimport) BOOL __stdcall ClientToScreen(HWND,void*);
__declspec(dllimport) LRESULT __stdcall GetWindowLongW(HWND,INT);
__declspec(dllimport) LRESULT __stdcall SetWindowLongW(HWND,INT,LRESULT);
__declspec(dllimport) BOOL __stdcall SetWindowPos(HWND,HWND,INT,INT,INT,INT,UINT);
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

typedef float FLOAT;typedef IDirect3DVertexBuffer9* LPDIRECT3DVERTEXBUFFER9;
#define D3DFVF_XYZRHW 4
#define D3DFVF_DIFFUSE 0x40
#define D3DPOOL_DEFAULT 0
#define D3DPT_TRIANGLELIST 4
#define WM_QUIT 18
#define PM_REMOVE 1
extern "C" __declspec(dllimport) BOOL __stdcall PeekMessageW(MSG*,HWND,UINT,UINT,UINT);
#define PeekMessage PeekMessageW
extern "C" void* memcpy(void*,const void*,UINT);
struct IDirect3DVertexBuffer9 {
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
 virtual HRESULT __stdcall Lock(UINT,UINT,void**,DWORD)=0;
 virtual HRESULT __stdcall Unlock()=0;
 virtual HRESULT __stdcall GetDesc(void*)=0;
};

struct IDirect3DVertexShader9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual HRESULT __stdcall GetDevice(IDirect3DDevice9**)=0;
 virtual HRESULT __stdcall GetFunction(void*,UINT*)=0;
};
struct IDirect3DPixelShader9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual HRESULT __stdcall GetDevice(IDirect3DDevice9**)=0;
 virtual HRESULT __stdcall GetFunction(void*,UINT*)=0;
};
struct IDirect3DVertexDeclaration9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual HRESULT __stdcall GetDevice(IDirect3DDevice9**)=0;
 virtual HRESULT __stdcall GetDeclaration(D3DVERTEXELEMENT9*,UINT*)=0;
};

struct IDirect3DBaseTexture9 {
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
 virtual void __stdcall UnsupportedSlot12()=0;
 virtual UINT __stdcall GetLevelCount()=0;
 virtual void __stdcall UnsupportedSlot14()=0;
 virtual void __stdcall UnsupportedSlot15()=0;
 virtual void __stdcall UnsupportedSlot16()=0;
};
struct IDirect3DTexture9 : IDirect3DBaseTexture9 {
 virtual HRESULT __stdcall GetLevelDesc(UINT,D3DSURFACE_DESC*)=0;
 virtual HRESULT __stdcall GetSurfaceLevel(UINT,IDirect3DSurface9**)=0;
 virtual HRESULT __stdcall LockRect(UINT,D3DLOCKED_RECT*,const void*,DWORD)=0;
 virtual HRESULT __stdcall UnlockRect(UINT)=0;
 virtual void __stdcall UnsupportedSlot21()=0;
};

struct D3DINDEXBUFFER_DESC {UINT Format,Type,Usage,Pool,Size;};
struct IDirect3DIndexBuffer9 {
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
 virtual HRESULT __stdcall Lock(UINT,UINT,void**,DWORD)=0;
 virtual HRESULT __stdcall Unlock()=0;
 virtual HRESULT __stdcall GetDesc(D3DINDEXBUFFER_DESC*)=0;
};

struct IDirect3DSwapChain9 {
 virtual HRESULT __stdcall QueryInterface(const GUID&,void**)=0;
 virtual ULONG __stdcall AddRef()=0;
 virtual ULONG __stdcall Release()=0;
 virtual HRESULT __stdcall Present(const void*,const void*,HWND,const void*,DWORD)=0;
 virtual HRESULT __stdcall GetFrontBufferData(IDirect3DSurface9*)=0;
 virtual HRESULT __stdcall GetBackBuffer(UINT,UINT,IDirect3DSurface9**)=0;
 virtual HRESULT __stdcall GetRasterStatus(void*)=0;
 virtual HRESULT __stdcall GetDisplayMode(D3DDISPLAYMODE*)=0;
 virtual HRESULT __stdcall GetDevice(IDirect3DDevice9**)=0;
 virtual HRESULT __stdcall GetPresentParameters(D3DPRESENT_PARAMETERS*)=0;
};
