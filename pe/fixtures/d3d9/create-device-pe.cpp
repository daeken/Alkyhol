// Original MIT witness around the byte-for-byte Microsoft tutorial. The original
// InitD3D, Render and Cleanup bodies run; only the interactive entry/loop is replaced.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
}
extern "C" { int _fltused=0; } // Freestanding COFF compiler marker, not a runtime service.
static DWORD words[256],used;
static void put(DWORD n){words[used++]=n;}
static DWORD stack(){DWORD s;__asm__ volatile("movl %%esp,%0":"=r"(s));return s;}
static LRESULT __stdcall witness_proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
extern "C" DWORD entry(){
 DWORD before=stack();put(0x44394433);put(4);
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=witness_proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"Alkyhol D3D9 witness";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Microsoft D3D9 Tutorial 01",0x80000000,80,80,65,3,NULL,NULL,wc.hInstance,NULL);
 if(!window)return 3;
 HRESULT hr=InitD3D(window);put(hr);if(FAILED(hr))return 4;
 put(g_pD3D->GetAdapterCount()>0);
 static DWORD adapter[277];adapter[0]=adapter[276]=0x715e7a11;put(g_pD3D->GetAdapterIdentifier(0,0,adapter+1)==S_OK&&adapter[0]==0x715e7a11&&adapter[276]==0x715e7a11&&adapter[1]!=0);
 const GUID unknown={0,0,0,{0xc0,0,0,0,0,0,0,0x46}};void* identity=NULL;
 put(g_pD3D->QueryInterface(unknown,&identity)==S_OK&&identity==g_pD3D);if(identity)((IDirect3D9*)identity)->Release();
 IDirect3D9* parent=NULL;put(g_pd3dDevice->GetDirect3D(&parent)==S_OK&&parent==g_pD3D);if(parent)parent->Release();
 put(g_pd3dDevice->TestCooperativeLevel()==S_OK);
 ShowWindow(window,1);Render(); // Original Microsoft clear/scene/present body.
 IDirect3DSurface9 *back=NULL,*copy=NULL;
 put(g_pd3dDevice->GetBackBuffer(0,0,D3DBACKBUFFER_TYPE_MONO,&back)==S_OK);if(!back)return 5;
 D3DSURFACE_DESC desc={};put(back->GetDesc(&desc)==S_OK&&desc.Width==65&&desc.Height==3&&desc.Format==D3DFMT_X8R8G8B8);
 put(g_pd3dDevice->CreateOffscreenPlainSurface(65,3,D3DFMT_X8R8G8B8,D3DPOOL_SYSTEMMEM,&copy,NULL)==S_OK);if(!copy)return 6;
 put(g_pd3dDevice->GetRenderTargetData(back,copy)==S_OK);
 D3DLOCKED_RECT locked={};put(copy->LockRect(&locked,NULL,D3DLOCK_READONLY)==S_OK);if(!locked.pBits)return 7;
 for(UINT y=0;y<3;y++)for(UINT x=0;x<65;x++)put(((DWORD*)((unsigned char*)locked.pBits+y*locked.Pitch))[x]&0xffffff);
 put(copy->UnlockRect()==S_OK);copy->Release();back->Release();
 put(g_pd3dDevice->BeginScene()==S_OK);put(g_pd3dDevice->BeginScene()==(HRESULT)0x8876086c);put(g_pd3dDevice->EndScene()==S_OK);put(g_pd3dDevice->EndScene()==(HRESULT)0x8876086c);
 Cleanup();g_pd3dDevice=NULL;g_pD3D=NULL;
 put(DestroyWindow(window)!=0);put(UnregisterClassW(wc.lpszClassName,wc.hInstance)!=0);put(stack()==before);put(used+1);
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,used*4,&done,NULL)||done!=used*4)return 8;
 return 0;
}
