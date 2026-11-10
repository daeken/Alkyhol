// Alkyhol MIT: advertised color render targets must have actual GPU backing.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
int _fltused=0;
}
static DWORD stack(){DWORD n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static LRESULT __stdcall proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
static DWORD witness(){
 IDirect3DSurface9* original=NULL;if(g_pd3dDevice->GetRenderTarget(0,&original)!=S_OK)return 1;
 for(UINT format=21;format<=22;format++){
  if(g_pD3D->CheckDeviceFormat(0,1,22,1,1,format)!=S_OK)return 2;
  if(g_pD3D->CheckDeviceFormat(0,1,22,1,3,format)!=S_OK)return 3;
  IDirect3DTexture9* texture=NULL;IDirect3DSurface9* surface=NULL;
  if(g_pd3dDevice->CreateTexture(16,16,1,1,format,0,&texture,NULL)!=S_OK||!texture)return 4;
  if(texture->GetSurfaceLevel(0,&surface)!=S_OK||!surface)return 5;
  D3DSURFACE_DESC desc={};if(surface->GetDesc(&desc)!=S_OK||desc.Width!=16||desc.Height!=16||desc.Format!=format)return 6;
  if(g_pd3dDevice->SetRenderTarget(0,surface)!=S_OK||g_pd3dDevice->Clear(0,NULL,1,0xff123456,1,0)!=S_OK)return 7;
  if(g_pd3dDevice->SetRenderTarget(0,original)!=S_OK)return 8;
  surface->Release();texture->Release();
  IDirect3DSurface9* independent=NULL;
  if(g_pd3dDevice->CreateRenderTarget(16,16,format,0,0,0,&independent,NULL)!=S_OK||!independent)return 9;
  if(g_pd3dDevice->SetRenderTarget(0,independent)!=S_OK||g_pd3dDevice->Clear(0,NULL,1,0xff654321,1,0)!=S_OK)return 10;
  if(g_pd3dDevice->SetRenderTarget(0,original)!=S_OK)return 11;
  independent->Release();
 }
 original->Release();return 0;
}
extern "C" DWORD entry(){
 DWORD before=stack(),words[8]={0x52544350,1,0,0,0,0,4,8};
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"AlkyholRenderCaps";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Alkyhol real render target caps",0x80000000,80,80,32,32,NULL,NULL,wc.hInstance,NULL);
 if(!window||InitD3D(window)!=S_OK)return 3;
 words[2]=witness();Cleanup();words[3]=DestroyWindow(window)!=0;words[4]=UnregisterClassW(wc.lpszClassName,wc.hInstance)!=0;words[5]=stack()==before;
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,sizeof(words),&done,NULL)||done!=sizeof(words))return 4;return words[2];
}
