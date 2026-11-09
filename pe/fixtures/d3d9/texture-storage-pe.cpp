// Original MIT witness for DAO's CPU texture storage and mapping rollover.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
int _fltused=0;
}
static DWORD stack(){DWORD n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static LRESULT __stdcall proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
static DWORD value(UINT n,UINT mip,UINT x,UINT y){return 0xff000000|(n<<12)|(mip<<8)|(y<<4)|x;}
static DWORD witness(){
 static IDirect3DTexture9* textures[768];
 for(UINT n=0;n<768;n++){
  if(g_pd3dDevice->CreateTexture(8,8,0,0,21,1,textures+n,NULL)!=S_OK||!textures[n]||textures[n]->GetLevelCount()!=4)return 1;
  for(UINT mip=0;mip<4;mip++){
   D3DSURFACE_DESC desc={};D3DLOCKED_RECT lock={};UINT edge=8>>mip;
   if(textures[n]->GetLevelDesc(mip,&desc)!=S_OK||desc.Width!=edge||desc.Height!=edge||desc.Pool!=1||desc.Format!=21)return 2;
   if(textures[n]->LockRect(mip,&lock,NULL,0)!=S_OK||!lock.pBits||lock.Pitch<(INT)(edge*4))return 3;
   for(UINT y=0;y<edge;y++)for(UINT x=0;x<edge;x++)((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]=value(n,mip,x,y);
   if(textures[n]->UnlockRect(mip)!=S_OK)return 4;
  }
 }
 for(UINT n=0;n<768;n++)for(UINT mip=0;mip<4;mip++){
  D3DLOCKED_RECT lock={};if(textures[n]->LockRect(mip,&lock,NULL,16)!=S_OK||!lock.pBits)return 5;
  for(UINT y=0;y<(8u>>mip);y++)for(UINT x=0;x<(8u>>mip);x++)if(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]!=value(n,mip,x,y))return 6;
  if(textures[n]->UnlockRect(mip)!=S_OK)return 7;
 }
 IDirect3DSurface9* alias=NULL;if(textures[767]->GetSurfaceLevel(2,&alias)!=S_OK||!alias||g_pd3dDevice->SetTexture(0,textures[767])!=S_OK)return 8;
 for(UINT n=0;n<768;n++)textures[n]->Release();
 IDirect3DBaseTexture9* held=NULL;if(g_pd3dDevice->GetTexture(0,&held)!=S_OK||held!=textures[767])return 9;
 // Later allocations cannot overwrite a binding or an externally held alias.
 for(UINT n=0;n<128;n++){
  IDirect3DTexture9* next=NULL;D3DLOCKED_RECT lock={};
  if(g_pd3dDevice->CreateTexture(8,8,1,0,21,1,&next,NULL)!=S_OK||!next||next->LockRect(0,&lock,NULL,0)!=S_OK||!lock.pBits)return 10;
  for(UINT y=0;y<8;y++)for(UINT x=0;x<8;x++)((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]=0xff123456;
  if(next->UnlockRect(0)!=S_OK)return 11;next->Release();
 }
 D3DLOCKED_RECT lock={};if(alias->LockRect(&lock,NULL,16)!=S_OK||!lock.pBits)return 12;
 for(UINT y=0;y<2;y++)for(UINT x=0;x<2;x++)if(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]!=value(767,2,x,y))return 13;
 if(alias->UnlockRect()!=S_OK||g_pd3dDevice->SetTexture(0,NULL)!=S_OK)return 14;
 held->Release();alias->Release();return 0;
}
extern "C" DWORD entry(){
 DWORD before=stack(),words[8]={0x54455841,1,0,0,0,0,768,8};
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"AlkyholTextureStorage";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Alkyhol texture allocation witness",0x80000000,80,80,32,32,NULL,NULL,wc.hInstance,NULL);
 if(!window||InitD3D(window)!=S_OK)return 3;
 words[2]=witness();Cleanup();words[3]=DestroyWindow(window)!=0;words[4]=UnregisterClassW(wc.lpszClassName,wc.hInstance)!=0;words[5]=stack()==before;
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,sizeof(words),&done,NULL)||done!=sizeof(words))return 4;return words[2];
}
