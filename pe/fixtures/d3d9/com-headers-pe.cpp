// Original MIT stress witness for DAO's actual small COM-header allocation limit.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
int _fltused=0;
}
static IDirect3DVertexDeclaration9* declarations[4094];
static const D3DVERTEXELEMENT9 elements[]={{0,0,3,0,0,0},{255,0,17,0,0,0}};
static DWORD stack(){DWORD n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static LRESULT __stdcall proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
static DWORD witness(){
 for(UINT cycle=0;cycle<2;cycle++){
  for(UINT n=0;n<4094;n++)if(g_pd3dDevice->CreateVertexDeclaration(elements,declarations+n)!=S_OK||!declarations[n])return 1;
  for(UINT n=0;n<4094;n++){
   struct {DWORD before;D3DVERTEXELEMENT9 data[2];DWORD after;} copy={};copy.before=copy.after=0x719bad15;UINT count=2;
   if(declarations[n]->GetDeclaration(copy.data,&count)!=S_OK||count!=2||copy.before!=0x719bad15||copy.after!=0x719bad15)return 2;
   const unsigned char* a=(const unsigned char*)elements;const unsigned char* b=(const unsigned char*)copy.data;
   for(UINT i=0;i<sizeof(elements);i++)if(a[i]!=b[i])return 3;
  }
  if(g_pd3dDevice->SetVertexDeclaration(declarations[4093])!=S_OK)return 4;
  for(UINT n=0;n<4094;n++)declarations[n]->Release();
  IDirect3DVertexDeclaration9* held=NULL;
  if(g_pd3dDevice->GetVertexDeclaration(&held)!=S_OK||held!=declarations[4093])return 5;
  IDirect3DDevice9* owner=NULL;if(held->GetDevice(&owner)!=S_OK||owner!=g_pd3dDevice)return 6;owner->Release();
  held->Release();if(g_pd3dDevice->SetVertexDeclaration(NULL)!=S_OK)return 7;
 }
 return 0;
}
extern "C" DWORD entry(){
 DWORD before=stack(),words[8]={0x434f4d48,1,0,0,0,0,2,8};
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"AlkyholComHeaders";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Alkyhol COM allocation witness",0x80000000,80,80,32,32,NULL,NULL,wc.hInstance,NULL);
 if(!window||InitD3D(window)!=S_OK)return 3;
 words[2]=witness();Cleanup();words[3]=DestroyWindow(window)!=0;words[4]=UnregisterClassW(wc.lpszClassName,wc.hInstance)!=0;words[5]=stack()==before;
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,sizeof(words),&done,NULL)||done!=sizeof(words))return 4;return words[2];
}
