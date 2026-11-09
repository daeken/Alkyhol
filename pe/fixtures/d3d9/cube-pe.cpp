// Original MIT finite resource witness for DAO's actual cube allocation path.
// Original Microsoft device setup remains verbatim; no game code or assets.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
int _fltused=0;
}
static const GUID cube_iid={0xfff32f81,0xd953,0x473a,{0x92,0x23,0x93,0xd6,0x52,0xab,0xa9,0x3f}};
static DWORD stack(){DWORD s;__asm__ volatile("movl %%esp,%0":"=r"(s));return s;}
static LRESULT __stdcall proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
static DWORD value(UINT face,UINT mip,UINT x,UINT y){return 0xff000000|(face<<16)|(mip<<8)|(x+y*4);}
static DWORD witness(){
 IDirect3DCubeTexture9* cube=NULL;
 #define REQUIRE(test,stage) if(!(test))return stage
 REQUIRE(g_pd3dDevice->CreateCubeTexture(4,0,0,21,1,&cube,NULL)==S_OK&&cube,1);
 REQUIRE(cube->GetType()==5&&cube->GetLevelCount()==3,2);
 void* identity=NULL;REQUIRE(cube->QueryInterface(cube_iid,&identity)==S_OK&&identity==cube,3);((IDirect3DCubeTexture9*)identity)->Release();
 IDirect3DDevice9* owner=NULL;REQUIRE(cube->GetDevice(&owner)==S_OK&&owner==g_pd3dDevice,4);owner->Release();
 for(UINT mip=0;mip<3;mip++){
  D3DSURFACE_DESC desc={};UINT edge=4>>mip;
  REQUIRE(cube->GetLevelDesc(mip,&desc)==S_OK&&desc.Type==1&&desc.Format==21&&desc.Pool==1&&desc.Usage==0&&desc.Width==edge&&desc.Height==edge,5);
  for(UINT face=0;face<6;face++){
   D3DLOCKED_RECT lock={};REQUIRE(cube->LockRect(face,mip,&lock,NULL,0)==S_OK&&lock.pBits&&lock.Pitch>=(INT)(edge*4),6);
   for(UINT y=0;y<edge;y++)for(UINT x=0;x<edge;x++)((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]=value(face,mip,x,y);
   REQUIRE(cube->UnlockRect(face,mip)==S_OK,7);
  }
 }
 for(UINT face=0;face<6;face++)for(UINT mip=0;mip<3;mip++){
  UINT edge=4>>mip;D3DLOCKED_RECT lock={};REQUIRE(cube->LockRect(face,mip,&lock,NULL,16)==S_OK&&lock.pBits,8);
  for(UINT y=0;y<edge;y++)for(UINT x=0;x<edge;x++)REQUIRE(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]==value(face,mip,x,y),9);
  REQUIRE(cube->UnlockRect(face,mip)==S_OK,10);
 }
 for(UINT face=0;face<6;face++){
  IDirect3DSurface9 *surface=NULL,*again=NULL;REQUIRE(cube->GetCubeMapSurface(face,1,&surface)==S_OK&&surface,11);
  REQUIRE(cube->GetCubeMapSurface(face,1,&again)==S_OK&&again==surface,12);again->Release();
  D3DSURFACE_DESC desc={};REQUIRE(surface->GetDesc(&desc)==S_OK&&desc.Width==2&&desc.Height==2&&desc.Format==21&&desc.Pool==1,13);
  identity=NULL;REQUIRE(surface->GetContainer(cube_iid,&identity)==S_OK&&identity==cube,14);((IDirect3DCubeTexture9*)identity)->Release();
  D3DLOCKED_RECT lock={};REQUIRE(surface->LockRect(&lock,NULL,16)==S_OK&&lock.pBits,15);
  for(UINT y=0;y<2;y++)for(UINT x=0;x<2;x++)REQUIRE(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]==value(face,1,x,y),16);
  REQUIRE(surface->UnlockRect()==S_OK,17);surface->Release();
 }
 D3DLOCKED_RECT invalid={};D3DSURFACE_DESC desc={};
 REQUIRE(FAILED(cube->LockRect(6,0,&invalid,NULL,0))&&FAILED(cube->LockRect(0,3,&invalid,NULL,0))&&FAILED(cube->GetLevelDesc(3,&desc)),18);
 REQUIRE(g_pd3dDevice->SetTexture(0,cube)==S_OK,19);cube->Release();
 IDirect3DBaseTexture9* retained=NULL;REQUIRE(g_pd3dDevice->GetTexture(0,&retained)==S_OK&&retained==cube&&retained->GetType()==5,20);
 REQUIRE(g_pd3dDevice->SetTexture(0,NULL)==S_OK,21);retained->Release();
 #undef REQUIRE
 return 0;
}
extern "C" DWORD entry(){
 DWORD before=stack(),words[8]={0x43554245,1,0,0,0,0,0,8};
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"AlkyholCubeWitness";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Alkyhol cube resource witness",0x80000000,80,80,32,32,NULL,NULL,wc.hInstance,NULL);
 if(!window||InitD3D(window)!=S_OK)return 3;
 words[2]=witness();Cleanup();words[3]=DestroyWindow(window)!=0;words[4]=UnregisterClassW(wc.lpszClassName,wc.hInstance)!=0;words[5]=stack()==before;words[6]=1;
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,sizeof(words),&done,NULL)||done!=sizeof(words))return 4;return words[2];
}
