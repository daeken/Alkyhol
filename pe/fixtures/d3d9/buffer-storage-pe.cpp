// Original MIT witness for DAO's actual vertex/index CPU-storage mapping limit.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
int _fltused=0;
}
static DWORD stack(){DWORD n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static LRESULT __stdcall proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
template<class Buffer,class Desc,bool Index> static DWORD batch(){
 static Buffer* buffers[4094];
 for(UINT n=0;n<4094;n++){
  HRESULT hr;
  if constexpr(Index)hr=g_pd3dDevice->CreateIndexBuffer(1040,0,101,0,buffers+n,NULL);
  else hr=g_pd3dDevice->CreateVertexBuffer(1040,0,0,0,buffers+n,NULL);
  if(hr!=S_OK||!buffers[n])return 1;
  struct {DWORD before;Desc data;DWORD after;} descriptor={};descriptor.before=descriptor.after=0x719bad15;
  if(buffers[n]->GetDesc(&descriptor.data)!=S_OK||descriptor.before!=0x719bad15||descriptor.after!=0x719bad15||descriptor.data.Size!=1040||descriptor.data.Usage!=0||descriptor.data.Pool!=0||descriptor.data.Type!=(Index?7u:6u))return 2;
  void* data=NULL;if(buffers[n]->Lock(0,0,&data,0)!=S_OK||!data)return 3;
  for(UINT word=0;word<260;word++)((DWORD*)data)[word]=(Index?0x10000000u:0u)|(n<<12)|word;
  if(buffers[n]->Unlock()!=S_OK)return 4;
 }
 // Every allocation must retain its own bytes across later arena allocations.
 for(UINT n=0;n<4094;n++){
  void* data=NULL;if(buffers[n]->Lock(0,0,&data,16)!=S_OK||!data)return 5;
  for(UINT word=0;word<260;word++)if(((DWORD*)data)[word]!=((Index?0x10000000u:0u)|(n<<12)|word))return 6;
  if(buffers[n]->Unlock()!=S_OK)return 7;
 }
 if constexpr(Index){if(g_pd3dDevice->SetIndices(buffers[4093])!=S_OK)return 8;}
 else if(g_pd3dDevice->SetStreamSource(0,buffers[4093],0,16)!=S_OK)return 8;
 for(UINT n=0;n<4094;n++)buffers[n]->Release();
 Buffer* held=NULL;
 if constexpr(Index){if(g_pd3dDevice->GetIndices(&held)!=S_OK||held!=buffers[4093])return 9;}
 else {UINT offset=99,stride=99;if(g_pd3dDevice->GetStreamSource(0,&held,&offset,&stride)!=S_OK||held!=buffers[4093]||offset!=0||stride!=16)return 9;}
 IDirect3DDevice9* owner=NULL;if(held->GetDevice(&owner)!=S_OK||owner!=g_pd3dDevice)return 10;owner->Release();
 void* data=NULL;if(held->Lock(0,0,&data,16)!=S_OK||!data)return 11;
 for(UINT word=0;word<260;word++)if(((DWORD*)data)[word]!=((Index?0x10000000u:0u)|(4093u<<12)|word))return 12;
 if(held->Unlock()!=S_OK)return 13;held->Release();
 if constexpr(Index){if(g_pd3dDevice->SetIndices(NULL)!=S_OK)return 14;}
 else if(g_pd3dDevice->SetStreamSource(0,NULL,0,0)!=S_OK)return 14;
 return 0;
}
extern "C" DWORD entry(){
 DWORD before=stack(),words[8]={0x42554641,1,0,0,0,0,2,8};
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"AlkyholBufferStorage";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Alkyhol buffer allocation witness",0x80000000,80,80,32,32,NULL,NULL,wc.hInstance,NULL);
 if(!window||InitD3D(window)!=S_OK)return 3;
 words[2]=batch<IDirect3DVertexBuffer9,D3DVERTEXBUFFER_DESC,false>();
 if(!words[2]){DWORD result=batch<IDirect3DIndexBuffer9,D3DINDEXBUFFER_DESC,true>();if(result)words[2]=100+result;}
 Cleanup();words[3]=DestroyWindow(window)!=0;words[4]=UnregisterClassW(wc.lpszClassName,wc.hInstance)!=0;words[5]=stack()==before;
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,sizeof(words),&done,NULL)||done!=sizeof(words))return 4;return words[2];
}
