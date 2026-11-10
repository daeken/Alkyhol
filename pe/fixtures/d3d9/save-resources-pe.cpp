// Alkyhol MIT: real surface downsampling and dynamic BC1 thumbnail storage.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
int _fltused=0;
}
static DWORD stack(){DWORD n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static LRESULT __stdcall proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
struct R {int left,top,right,bottom;};
static DWORD witness(){
 IDirect3DTexture9* texture=NULL;
 if(g_pD3D->CheckDeviceFormat(0,1,22,0x200,3,0x31545844)!=S_OK)return 1;
 if(g_pd3dDevice->CreateTexture(4,4,1,0x200,0x31545844,0,&texture,NULL)!=S_OK||!texture)return 2;
 D3DSURFACE_DESC desc={};D3DLOCKED_RECT lock={};
 if(texture->GetLevelDesc(0,&desc)!=S_OK||desc.Usage!=0x200||desc.Pool!=0||desc.Format!=0x31545844)return 3;
 if(texture->LockRect(0,&lock,NULL,0x2000)!=S_OK||lock.Pitch!=8)return 4;
 ((DWORD*)lock.pBits)[0]=0x001ff800;((DWORD*)lock.pBits)[1]=0;
 if(texture->UnlockRect(0)!=S_OK||texture->LockRect(0,&lock,NULL,16)!=S_OK)return 5;
 if(((DWORD*)lock.pBits)[0]!=0x001ff800||((DWORD*)lock.pBits)[1]!=0)return 6;
 if(texture->UnlockRect(0)!=S_OK)return 7;texture->Release();
 IDirect3DSurface9 *source=NULL,*target=NULL,*cpu=NULL;
 if(g_pd3dDevice->CreateRenderTarget(4,4,21,0,0,0,&source,NULL)!=S_OK||g_pd3dDevice->CreateRenderTarget(4,4,21,0,0,1,&target,NULL)!=S_OK)return 8;
 R blue={2,0,4,4};if(g_pd3dDevice->ColorFill(source,NULL,0xffff0000)!=S_OK||g_pd3dDevice->ColorFill(source,&blue,0xff0000ff)!=S_OK||g_pd3dDevice->ColorFill(target,NULL,0xff00ff00)!=S_OK)return 9;
 R crop={0,0,4,4},dest={1,1,3,3};
 if(g_pd3dDevice->StretchRect(source,&crop,target,&dest,0)!=S_OK)return 10;
 if(g_pd3dDevice->CreateOffscreenPlainSurface(4,4,21,2,&cpu,NULL)!=S_OK||g_pd3dDevice->GetRenderTargetData(target,cpu)!=S_OK)return 11;
 if(cpu->LockRect(&lock,NULL,16)!=S_OK)return 12;
 for(UINT y=0;y<4;y++)for(UINT x=0;x<4;x++){
  DWORD expected=y>=1&&y<3&&x==1?0xffff0000:y>=1&&y<3&&x==2?0xff0000ff:0xff00ff00;
  if(*(DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch+x*4)!=expected)return 13;
 }
 if(cpu->UnlockRect()!=S_OK)return 14;
 if(target->LockRect(&lock,&dest,0x810)!=S_OK)return 15;
 if(((DWORD*)lock.pBits)[0]!=0xffff0000||((DWORD*)lock.pBits)[1]!=0xff0000ff)return 16;
 if(target->UnlockRect()!=S_OK)return 17;
 R corner={0,0,1,1};
 if(target->LockRect(&lock,&corner,0)!=S_OK)return 18;
 *(DWORD*)lock.pBits=0xffaabbcc;
 if(target->UnlockRect()!=S_OK||g_pd3dDevice->GetRenderTargetData(target,cpu)!=S_OK)return 19;
 if(cpu->LockRect(&lock,NULL,16)!=S_OK)return 20;
 for(UINT y=0;y<4;y++)for(UINT x=0;x<4;x++){
  DWORD expected=y==0&&x==0?0xffaabbcc:y>=1&&y<3&&x==1?0xffff0000:y>=1&&y<3&&x==2?0xff0000ff:0xff00ff00;
  if(*(DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch+x*4)!=expected)return 21;
 }
 if(cpu->UnlockRect()!=S_OK)return 22;
 cpu->Release();target->Release();source->Release();return 0;
}
extern "C" DWORD entry(){
 DWORD before=stack(),words[8]={0x53415645,2,0,0,0,0,3,8};
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"AlkyholSaveResources";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Alkyhol real save resources",0x80000000,80,80,32,32,NULL,NULL,wc.hInstance,NULL);
 if(!window||InitD3D(window)!=S_OK)return 3;
 words[2]=witness();Cleanup();words[3]=DestroyWindow(window)!=0;words[4]=UnregisterClassW(wc.lpszClassName,wc.hInstance)!=0;words[5]=stack()==before;
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,sizeof(words),&done,NULL)||done!=sizeof(words))return 4;return words[2];
}
