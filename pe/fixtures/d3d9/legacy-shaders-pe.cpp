// Owned MIT programmable shader witness. Windows and Alkyhol execute identical
// literal legacy bytecode; no native recompilation or replacement shader body.
#include "../../../samples/d3d9/create_device/CreateDevice.cpp"
extern "C" {
__declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
__declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
int _fltused=0;
}
static const DWORD vertex_code[]={
#ifdef MODEL2
#include "legacy-model2.vs.inc"
#else
#include "legacy-color.vs.inc"
#endif
};
static const DWORD pixel_code[]={
#ifdef MODEL2
#include "legacy-slots128.ps.inc"
#elif defined(TEXTURES)
#include "legacy-texture.ps.inc"
#else
#include "legacy-color.ps.inc"
#endif
};
static DWORD words[2300],used;
static void put(DWORD n){words[used++]=n;}
static void ok(BOOL n){put(n?1:0);}
static DWORD fail(DWORD status,HRESULT detail){put(0xdeadc0de);put(status);put((DWORD)detail);DWORD done=0;WriteFile(GetStdHandle((DWORD)-11),words,used*4,&done,NULL);return status;}
static DWORD stack(){DWORD s;__asm__ volatile("movl %%esp,%0":"=r"(s));return s;}
static LRESULT __stdcall proc(HWND w,UINT m,WPARAM a,LPARAM b){return DefWindowProcW(w,m,a,b);}
#ifdef TRANSFORMED
// Actual screen coordinates and nonuniform RHW. The bound programmable VS must
// be bypassed by POSITIONT; original PS samples the interpolated COLOR0.xy.
static const float vertices[][8]={{4,4,.5f,.5f,0,0,0,1},{28,4,.5f,2,1,0,0,1},{4,28,.5f,.5f,0,1,0,1},{28,28,.5f,2,1,1,0,1}};
static float matrix[16]={1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1};
static const D3DVERTEXELEMENT9 elements[]={{0,0,3,0,9,0},{0,16,3,0,10,0},{255,0,17,0,0,0}};
#elif !defined(PACKED)
static const float vertices[][8]={{0,.75f,.5f,1,1,0,0,1},{.75f,-.75f,.5f,1,1,0,0,1},{-.75f,-.75f,.5f,1,1,0,0,1}};
static float matrix[16]={1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1};
static const D3DVERTEXELEMENT9 elements[]={{0,0,3,0,0,0},{0,16,3,0,10,0},{255,0,17,0,0,0}};
#else
struct PackedVertex { short x,y; DWORD color; };
static const PackedVertex vertices[]={{0,1,0xffff0000},{1,-1,0xffff0000},{-1,-1,0xffff0000}};
static float matrix[16]={.75f,0,0,0,0,.75f,0,0,0,0,1,0,0,0,0,1};
static const D3DVERTEXELEMENT9 elements[]={{0,0,6,0,0,0},{0,4,4,0,10,0},{255,0,17,0,0,0}};
#endif

#ifdef TARGETS
// Render, read back and sample one allocation. External texture/surface release
// must not destroy bound storage or its cached COM identities.
static DWORD render_target_witness(IDirect3DSurface9* back){
 IDirect3DTexture9* texture=NULL;IDirect3DSurface9 *surface=NULL,*again=NULL,*copy=NULL;
 const GUID texture_iid={0x85c31227,0x3de5,0x4f00,{0x9b,0x3a,0xf1,0x1a,0xc3,0x8c,0x18,0xb5}};
 #define REQUIRE_TARGET(test,stage) if(!(test))return stage
 REQUIRE_TARGET(g_pd3dDevice->CreateTexture(8,8,1,1,21,0,&texture,NULL)==S_OK&&texture,1);
 D3DSURFACE_DESC desc={};REQUIRE_TARGET(texture->GetLevelDesc(0,&desc)==S_OK&&desc.Usage==1&&desc.Pool==0&&desc.Width==8&&desc.Height==8,2);
 REQUIRE_TARGET(texture->GetSurfaceLevel(0,&surface)==S_OK&&surface,3);
 REQUIRE_TARGET(texture->GetSurfaceLevel(0,&again)==S_OK&&again==surface,4);again->Release();
 REQUIRE_TARGET(g_pd3dDevice->SetRenderTarget(0,surface)==S_OK,5);
 REQUIRE_TARGET(g_pd3dDevice->GetRenderTarget(0,&again)==S_OK&&again==surface,6);again->Release();
 DWORD viewport[6]={};REQUIRE_TARGET(g_pd3dDevice->GetViewport(viewport)==S_OK&&viewport[2]==8&&viewport[3]==8,7);
 REQUIRE_TARGET(g_pd3dDevice->Clear(0,NULL,1,0x80808080,1,0)==S_OK,8);
 REQUIRE_TARGET(g_pd3dDevice->CreateOffscreenPlainSurface(8,8,21,2,&copy,NULL)==S_OK&&copy,9);
 REQUIRE_TARGET(g_pd3dDevice->GetRenderTargetData(surface,copy)==S_OK,10);
 D3DLOCKED_RECT lock={};REQUIRE_TARGET(copy->LockRect(&lock,NULL,16)==S_OK&&lock.pBits,11);
 for(UINT y=0;y<8;y++)for(UINT x=0;x<8;x++)REQUIRE_TARGET(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]==0x80808080,12);
 REQUIRE_TARGET(copy->UnlockRect()==S_OK,13);copy->Release();
 texture->Release(); // Only the external surface and binding remain.
 void* container=NULL;REQUIRE_TARGET(surface->GetContainer(texture_iid,&container)==S_OK&&container==texture,14);
 REQUIRE_TARGET(g_pd3dDevice->SetTexture(0,(IDirect3DTexture9*)container)==S_OK,15);((IDirect3DTexture9*)container)->Release();
 surface->Release(); // Only internal render/sampler bindings remain.
 REQUIRE_TARGET(g_pd3dDevice->GetRenderTarget(0,&again)==S_OK&&again==surface,16);
 REQUIRE_TARGET(g_pd3dDevice->SetRenderTarget(0,back)==S_OK,17);again->Release();
 REQUIRE_TARGET(g_pd3dDevice->GetViewport(viewport)==S_OK&&viewport[2]==32&&viewport[3]==32,18);
 IDirect3DBaseTexture9* retained=NULL;REQUIRE_TARGET(g_pd3dDevice->GetTexture(0,&retained)==S_OK&&retained==texture,19);
 REQUIRE_TARGET(((IDirect3DTexture9*)retained)->GetSurfaceLevel(0,&again)==S_OK&&again==surface,20);again->Release();retained->Release();
 #undef REQUIRE_TARGET
 return 0;
}
#endif
#ifdef DEPTH
static DWORD depth_witness(){
 IDirect3DSurface9 *automatic=NULL,*surface=NULL,*again=NULL;D3DSURFACE_DESC desc={};
 #define REQUIRE_DEPTH(test,stage) if(!(test))return stage
 REQUIRE_DEPTH(g_pD3D->CheckDeviceFormat(0,1,22,2,1,80)==S_OK,1);
 REQUIRE_DEPTH(g_pd3dDevice->GetDepthStencilSurface(&automatic)==S_OK&&automatic,2);
 REQUIRE_DEPTH(automatic->GetDesc(&desc)==S_OK&&desc.Format==80&&desc.Usage==2&&desc.Pool==0&&desc.Width==32&&desc.Height==32,3);
 REQUIRE_DEPTH(g_pd3dDevice->CreateDepthStencilSurface(32,32,80,0,0,TRUE,&surface,NULL)==S_OK&&surface,4);
 REQUIRE_DEPTH(g_pd3dDevice->SetDepthStencilSurface(surface)==S_OK,5);
 automatic->Release();surface->Release();
 REQUIRE_DEPTH(g_pd3dDevice->GetDepthStencilSurface(&again)==S_OK&&again==surface,6);
 REQUIRE_DEPTH(g_pd3dDevice->SetDepthStencilSurface(NULL)==S_OK,7);
 IDirect3DSurface9* absent=(IDirect3DSurface9*)1;REQUIRE_DEPTH(g_pd3dDevice->GetDepthStencilSurface(&absent)==(HRESULT)0x88760866&&absent==NULL,8);
 REQUIRE_DEPTH(g_pd3dDevice->SetDepthStencilSurface(again)==S_OK,9);again->Release();
 DWORD enabled=0;REQUIRE_DEPTH(g_pd3dDevice->GetRenderState(7,&enabled)==S_OK&&enabled==1,10);
 REQUIRE_DEPTH(g_pd3dDevice->SetRenderState(23,4)==S_OK,11);
 #undef REQUIRE_DEPTH
 return 0;
}
#endif
#ifdef INDEXED
static DWORD index_witness(){
 IDirect3DIndexBuffer9 *buffer=NULL,*again=NULL;void* data=NULL;D3DINDEXBUFFER_DESC desc={};
 #define REQUIRE_INDEX(test,stage) if(!(test))return stage
 REQUIRE_INDEX(g_pd3dDevice->CreateIndexBuffer(8,8,101,0,&buffer,NULL)==S_OK&&buffer,1);
 REQUIRE_INDEX(buffer->GetDesc(&desc)==S_OK&&desc.Format==101&&desc.Type==7&&desc.Usage==8&&desc.Pool==0&&desc.Size==8&&buffer->GetType()==7,2);
 IDirect3DDevice9* owner=NULL;REQUIRE_INDEX(buffer->GetDevice(&owner)==S_OK&&owner==g_pd3dDevice,3);owner->Release();
 REQUIRE_INDEX(buffer->Lock(0,0,&data,0)==S_OK&&data,4);unsigned short values[4]={0xdead,2,3,4};for(UINT i=0;i<4;i++)((unsigned short*)data)[i]=values[i];
 REQUIRE_INDEX(buffer->Unlock()==S_OK,5);REQUIRE_INDEX(g_pd3dDevice->SetIndices(buffer)==S_OK,6);buffer->Release();
 REQUIRE_INDEX(g_pd3dDevice->GetIndices(&again)==S_OK&&again==buffer,7);again->Release();
 IDirect3DVertexBuffer9* stream=NULL;DWORD offset=0,stride=0;REQUIRE_INDEX(g_pd3dDevice->GetStreamSource(0,&stream,&offset,&stride)==S_OK&&stream&&offset==32&&stride==32,8);stream->Release();
 #undef REQUIRE_INDEX
 return 0;
}
static DWORD wide_index_witness(){
 IDirect3DIndexBuffer9* buffer=NULL;void* data=NULL;D3DINDEXBUFFER_DESC desc={};
 #define REQUIRE_WIDE(test,stage) if(!(test))return stage
 REQUIRE_WIDE(g_pd3dDevice->CreateIndexBuffer(16,0x208,102,0,&buffer,NULL)==S_OK&&buffer,1);
 REQUIRE_WIDE(buffer->GetDesc(&desc)==S_OK&&desc.Format==102&&desc.Size==16&&desc.Usage==0x208,2);
 REQUIRE_WIDE(buffer->Lock(0,0,&data,0x2000)==S_OK&&data,3);DWORD values[4]={0xdeadbeef,2,3,4};for(UINT i=0;i<4;i++)((DWORD*)data)[i]=values[i];REQUIRE_WIDE(buffer->Unlock()==S_OK,4);
 REQUIRE_WIDE(buffer->Lock(0,4,&data,0x1000)==S_OK&&data,5);*(DWORD*)data=0xdeadbeee;REQUIRE_WIDE(buffer->Unlock()==S_OK,6);
 REQUIRE_WIDE(g_pd3dDevice->SetIndices(buffer)==S_OK,7);buffer->Release();
 #undef REQUIRE_WIDE
 return 0;
}
#endif
#ifdef BC3
static DWORD compressed_witness(){
 const DWORD format=0x35545844;IDirect3DTexture9* texture=NULL;
 #define REQUIRE_BC(test,stage) if(!(test))return stage
 REQUIRE_BC(g_pD3D->CheckDeviceFormat(0,1,22,0x80000,3,format)==S_OK,1);
 REQUIRE_BC(g_pd3dDevice->CreateTexture(8,8,0,0,format,1,&texture,NULL)==S_OK&&texture&&texture->GetLevelCount()==4,2);
 const unsigned char encoded[16]={128,255,0,0,0,0,0,0,0x10,0x84,0,0,0,0,0,0};
 const GUID texture_iid={0x85c31227,0x3de5,0x4f00,{0x9b,0x3a,0xf1,0x1a,0xc3,0x8c,0x18,0xb5}};
 for(UINT level=0;level<4;level++){
  IDirect3DSurface9 *surface=NULL,*again=NULL;REQUIRE_BC(texture->GetSurfaceLevel(level,&surface)==S_OK&&surface,3);
  REQUIRE_BC(texture->GetSurfaceLevel(level,&again)==S_OK&&again==surface,4);again->Release();
  UINT extent=8>>level;if(!extent)extent=1;D3DSURFACE_DESC desc={};REQUIRE_BC(surface->GetDesc(&desc)==S_OK&&desc.Format==format&&desc.Usage==0&&desc.Pool==1&&desc.Width==extent&&desc.Height==extent,5);
  void* container=NULL;REQUIRE_BC(surface->GetContainer(texture_iid,&container)==S_OK&&container==texture,6);((IDirect3DTexture9*)container)->Release();
  D3DLOCKED_RECT lock={};REQUIRE_BC(surface->LockRect(&lock,NULL,0)==S_OK&&lock.pBits&&lock.Pitch>=(INT)((extent+3)/4*16),7);
  for(UINT y=0;y<(extent+3)/4;y++)for(UINT x=0;x<(extent+3)/4;x++)for(UINT b=0;b<16;b++)((unsigned char*)lock.pBits)[y*lock.Pitch+x*16+b]=encoded[b];
  REQUIRE_BC(surface->UnlockRect()==S_OK,8);surface->Release();
  REQUIRE_BC(texture->GetSurfaceLevel(level,&again)==S_OK&&again==surface,9);
  REQUIRE_BC(again->LockRect(&lock,NULL,16)==S_OK&&lock.pBits,10);
  for(UINT y=0;y<(extent+3)/4;y++)for(UINT x=0;x<(extent+3)/4;x++)for(UINT b=0;b<16;b++)REQUIRE_BC(((unsigned char*)lock.pBits)[y*lock.Pitch+x*16+b]==encoded[b],11);
  REQUIRE_BC(again->UnlockRect()==S_OK,12);again->Release();
 }
 REQUIRE_BC(g_pd3dDevice->SetTexture(0,texture)==S_OK,13);texture->Release();
 #undef REQUIRE_BC
 return 0;
}
#endif
extern "C" DWORD entry(){
 DWORD before=stack();put(0x39534844);
#ifdef TRANSFORMED
put(11);
#elif defined(LUMINANCE)
put(10);
#elif defined(PACKED)
put(9);
#elif defined(BC3)
put(8);
#elif defined(INDEXED)
put(7);
#elif defined(DEPTH)
put(6);
#elif defined(TARGETS)
put(5);
#elif defined(STATES)
put(4);
#elif defined(MODEL2)
put(3);
#elif defined(TEXTURES)
put(2);
#else
put(1);
#endif
 WNDCLASSEX wc={};wc.cbSize=sizeof(wc);wc.lpfnWndProc=proc;wc.hInstance=GetModuleHandleW(NULL);wc.lpszClassName=L"Alkyhol legacy shaders";
 if(!RegisterClassExW(&wc))return 2;
 HWND window=CreateWindowExW(0,wc.lpszClassName,L"Original legacy shader execution",0x80000000,80,80,32,32,NULL,NULL,wc.hInstance,NULL);
 if(!window)return 3;
 #if defined(MODEL2)||defined(DEPTH)
 g_pD3D=Direct3DCreate9(32);if(!g_pD3D)return 4;
 D3DPRESENT_PARAMETERS present={};present.BackBufferWidth=32;present.BackBufferHeight=32;present.BackBufferFormat=21;present.BackBufferCount=1;present.SwapEffect=1;present.hDeviceWindow=window;present.Windowed=TRUE;present.PresentationInterval=0x80000000;
 #ifdef DEPTH
 present.EnableAutoDepthStencil=TRUE;present.AutoDepthStencilFormat=80;
 #endif
 if(g_pD3D->CreateDevice(0,1,window,0x24,&present,&g_pd3dDevice)!=S_OK)return 4;
 #else
 if(InitD3D(window)!=S_OK)return 4;
 #endif
 #ifdef MODEL2
 static DWORD caps[78];caps[0]=caps[77]=0x715e7a11;DWORD creation[4]={};
 ok(g_pD3D->GetDeviceCaps(0,1,caps+1)==S_OK&&caps[0]==0x715e7a11&&caps[77]==0x715e7a11&&caps[50]>=0xfffe0200&&caps[52]>=0xffff0200&&caps[71]>=128&&caps[23]>=32&&caps[24]>=32&&g_pd3dDevice->GetCreationParameters(creation)==S_OK&&creation[3]==0x24);
 #endif
 IDirect3DVertexShader9* vs=NULL;IDirect3DPixelShader9* ps=NULL;IDirect3DVertexDeclaration9* decl=NULL;IDirect3DVertexBuffer9* vb=NULL;
 HRESULT vertex_hr=g_pd3dDevice->CreateVertexShader(vertex_code,&vs);ok(vertex_hr==S_OK);if(!vs)return fail(5,vertex_hr);
 HRESULT pixel_hr=g_pd3dDevice->CreatePixelShader(pixel_code,&ps);ok(pixel_hr==S_OK);if(!ps)return fail(6,pixel_hr);
 UINT size=0;ok(vs->GetFunction(NULL,&size)==S_OK&&size==sizeof(vertex_code));
 static DWORD copied[256];ok(vs->GetFunction(copied,&size)==S_OK);BOOL exact=TRUE;for(UINT i=0;i<sizeof(vertex_code)/4;i++)if(copied[i]!=vertex_code[i])exact=FALSE;ok(exact);
 IDirect3DDevice9* parent=NULL;ok(vs->GetDevice(&parent)==S_OK&&parent==g_pd3dDevice);if(parent)parent->Release();
 ok(g_pd3dDevice->CreateVertexDeclaration(elements,&decl)==S_OK);if(!decl)return 7;
 UINT count=0;ok(decl->GetDeclaration(NULL,&count)==S_OK&&count==3);
 static D3DVERTEXELEMENT9 returned[3];ok(decl->GetDeclaration(returned,&count)==S_OK);exact=TRUE;for(UINT i=0;i<sizeof(elements);i++)if(((unsigned char*)returned)[i]!=((const unsigned char*)elements)[i])exact=FALSE;ok(exact);
 ok(g_pd3dDevice->SetVertexDeclaration(decl)==S_OK);ok(g_pd3dDevice->SetVertexShader(vs)==S_OK);ok(g_pd3dDevice->SetPixelShader(ps)==S_OK);
 #ifdef INDEXED
 const DWORD vb_size=sizeof(vertices)+64,vb_usage=0x208,vb_offset=64,stream_offset=32,lock_flags=0x2000;
 #else
 const DWORD vb_size=sizeof(vertices),vb_usage=8,vb_offset=0,stream_offset=0,lock_flags=0;
 #endif
 D3DVERTEXBUFFER_DESC vb_desc={};ok(g_pd3dDevice->CreateVertexBuffer(vb_size,vb_usage,0,0,&vb,NULL)==S_OK&&vb&&vb->GetDesc(&vb_desc)==S_OK&&vb_desc.Usage==vb_usage&&vb_desc.Size==vb_size);if(!vb)return 8;
 void* storage=NULL;ok(vb->Lock(0,0,&storage,lock_flags)==S_OK);if(!storage)return 9;
 for(UINT i=0;i<vb_size;i++)((unsigned char*)storage)[i]=0;for(UINT i=0;i<sizeof(vertices);i++)((unsigned char*)storage)[i+vb_offset]=((const unsigned char*)vertices)[i];ok(vb->Unlock()==S_OK);
 ok(g_pd3dDevice->SetStreamSource(0,vb,stream_offset,sizeof(vertices[0]))==S_OK);
 // Internal bindings retain resources, including shader bytecode and declaration.
 vs->Release();ps->Release();decl->Release();vb->Release();
 IDirect3DVertexShader9* bound=NULL;ok(g_pd3dDevice->GetVertexShader(&bound)==S_OK&&bound==vs);if(bound)bound->Release();
 IDirect3DSurface9 *back=NULL,*copy=NULL;ok(g_pd3dDevice->GetBackBuffer(0,0,0,&back)==S_OK);ok(g_pd3dDevice->CreateOffscreenPlainSurface(32,32,
#if defined(MODEL2)||defined(DEPTH)
21,
#else
22,
#endif
2,&copy,NULL)==S_OK);if(!back||!copy)return 10;
 #ifdef TEXTURES
 const UINT sampled_format=
 #ifdef LUMINANCE
 50;
 #else
 21;
 #endif
 #ifdef TRANSFORMED
 const UINT texture_extent=2;
 #else
 const UINT texture_extent=1;
 #endif
 IDirect3DTexture9* texture=NULL;ok(g_pd3dDevice->CreateTexture(texture_extent,texture_extent,1,0,sampled_format,1,&texture,NULL)==S_OK);if(!texture)return 13;
 #ifdef LUMINANCE
 D3DLOCKED_RECT initial={};BOOL initial_lock=texture->LockRect(0,&initial,NULL,16)==S_OK&&initial.pBits&&initial.Pitch>=1;
 if(initial_lock){volatile unsigned char observed=*(volatile unsigned char*)initial.pBits;(void)observed;initial_lock=texture->UnlockRect(0)==S_OK;}ok(initial_lock); // No initial byte value is promised or compared.
 #endif
 ok(texture->GetLevelCount()==1);D3DSURFACE_DESC desc={};ok(texture->GetLevelDesc(0,&desc)==S_OK&&desc.Format==sampled_format&&desc.Type==1&&desc.Pool==1&&desc.Width==texture_extent&&desc.Height==texture_extent);
 INT whole_texture[4]={0,0,(INT)texture_extent,(INT)texture_extent};D3DLOCKED_RECT texlock={};ok(texture->LockRect(0,&texlock,whole_texture,0)==S_OK);if(!texlock.pBits)return 14;
 #ifdef TRANSFORMED
 ((DWORD*)texlock.pBits)[0]=0xffff0000;((DWORD*)texlock.pBits)[1]=0xff00ff00;
 ((DWORD*)((unsigned char*)texlock.pBits+texlock.Pitch))[0]=0xff0000ff;((DWORD*)((unsigned char*)texlock.pBits+texlock.Pitch))[1]=0xffffffff;
 #elif defined(LUMINANCE)
 *(unsigned char*)texlock.pBits=128;
 #else
 *(DWORD*)texlock.pBits=0x80808080;
 #endif
 ok(texture->UnlockRect(0)==S_OK);ok(g_pd3dDevice->SetTexture(0,texture)==S_OK);
 IDirect3DBaseTexture9* retrieved=NULL;ok(g_pd3dDevice->GetTexture(0,&retrieved)==S_OK&&retrieved==texture);if(retrieved)retrieved->Release();
 parent=NULL;ok(texture->GetDevice(&parent)==S_OK&&parent==g_pd3dDevice);if(parent)parent->Release();ok(texture->GetType()==3);
 #ifdef TRANSFORMED
 const DWORD filter=1;
 #else
 const DWORD filter=2;
 #endif
 DWORD state=99;ok(g_pd3dDevice->GetSamplerState(0,11,&state)==S_OK&&state==0);ok(g_pd3dDevice->SetSamplerState(0,5,filter)==S_OK);ok(g_pd3dDevice->GetSamplerState(0,5,&state)==S_OK&&state==filter);
 ok(g_pD3D->CheckDeviceFormat(0,1,22,
 #ifdef LUMINANCE
 0,
 #else
 0x80000,
 #endif
 3,sampled_format)==S_OK);
 const GUID base_texture={0x580ca87e,0x1d3c,0x4d54,{0x99,0x1d,0xb7,0xd3,0xe3,0xc2,0x98,0xce}};void* texture_identity=NULL;
 ok(texture->QueryInterface(base_texture,&texture_identity)==S_OK&&texture_identity==texture);if(texture_identity)((IDirect3DBaseTexture9*)texture_identity)->Release();
 texture->Release();
 #ifdef TARGETS
 DWORD target_stage=render_target_witness(back);ok(target_stage==0);if(target_stage)return fail(16,target_stage);
 #endif
 #endif
 #ifdef BC3
 DWORD bc_stage=compressed_witness();ok(bc_stage==0);if(bc_stage)return fail(20,bc_stage);
 #endif
 #ifdef STATES
 ok(g_pd3dDevice->SetRenderState(27,1)==S_OK);ok(g_pd3dDevice->SetRenderState(19,5)==S_OK);ok(g_pd3dDevice->SetRenderState(20,6)==S_OK);
 DWORD frequency=0;ok(g_pd3dDevice->SetStreamSourceFreq(0,1)==S_OK&&g_pd3dDevice->GetStreamSourceFreq(0,&frequency)==S_OK&&frequency==1);
 IDirect3DIndexBuffer9* index=NULL;ok(g_pd3dDevice->SetIndices(NULL)==S_OK&&g_pd3dDevice->GetIndices(&index)==S_OK&&index==NULL);
 DWORD operation=0;ok(g_pd3dDevice->GetTextureStageState(0,1,&operation)==S_OK&&operation==4&&g_pd3dDevice->SetTextureStageState(0,1,1)==S_OK);
 ok(g_pd3dDevice->GetTextureStageState(0,1,&operation)==S_OK&&operation==1);
 DWORD blend=0;ok(g_pd3dDevice->GetRenderState(7,&blend)==S_OK&&blend==0&&g_pd3dDevice->GetRenderState(27,&blend)==S_OK&&blend==1);
 #endif
 #ifdef DEPTH
 DWORD depth_stage=depth_witness();ok(depth_stage==0);if(depth_stage)return fail(17,depth_stage);
 #endif
 #ifdef INDEXED
 DWORD index_stage=index_witness();ok(index_stage==0);if(index_stage)return fail(18,index_stage);
 #endif
 for(UINT frame=0;frame<2;frame++){
  #ifdef TRANSFORMED
  DWORD full[6]={0,0,32,32,0,0x3f800000};
  if(g_pd3dDevice->SetViewport(full)!=S_OK)return 26;
  #endif
  #ifdef PACKED
  // Change only the fetch representation on frame 2. The shader bodies and
  // original integer positions stay identical; UBYTE4N RGBA differs from ARGB.
  if(frame){
   D3DVERTEXELEMENT9 normalized[]={{0,0,6,0,0,0},{0,4,8,0,10,0},{255,0,17,0,0,0}};
   IDirect3DVertexDeclaration9* next=NULL;
   if(g_pd3dDevice->CreateVertexDeclaration(normalized,&next)!=S_OK||!next)return 21;
   if(g_pd3dDevice->SetVertexDeclaration(next)!=S_OK)return 22;next->Release();
   IDirect3DVertexBuffer9* retained=NULL;UINT offset=99,stride=99;
   if(g_pd3dDevice->GetStreamSource(0,&retained,&offset,&stride)!=S_OK||retained!=vb||offset!=0||stride!=8)return 23;
   void* packed=NULL;if(retained->Lock(0,0,&packed,0)!=S_OK||!packed)return 24;
   for(UINT i=0;i<3;i++)((PackedVertex*)packed)[i].color=0xff0000ff;
   if(retained->Unlock()!=S_OK)return 25;retained->Release();
  }
  #endif

  #ifdef INDEXED
  if(frame){DWORD stage=wide_index_witness();if(stage)return fail(19,stage);}
  #endif
  matrix[3]=frame?.75f:0;
  put(0xfeed0000+frame);
  #ifdef STATES
  ok(g_pd3dDevice->SetRenderState(162,frame?0xffffffffu:0)==S_OK);DWORD mask=1;ok(g_pd3dDevice->GetRenderState(162,&mask)==S_OK&&mask==(frame?0xffffffffu:0));
  ok(g_pd3dDevice->SetRenderState(168,frame?1:15)==S_OK);DWORD writes=0;ok(g_pd3dDevice->GetRenderState(168,&writes)==S_OK&&writes==(frame?1u:15u));
  #endif
  #ifdef TEXTURES
  const DWORD srgb_value=
  #ifdef LUMINANCE
  0;
  #else
  frame;
  #endif
  ok(g_pd3dDevice->SetSamplerState(0,11,srgb_value)==S_OK);DWORD srgb=99;ok(g_pd3dDevice->GetSamplerState(0,11,&srgb)==S_OK&&srgb==srgb_value);
  #endif
  ok(g_pd3dDevice->SetVertexShaderConstantF(0,matrix,4)==S_OK);
  static float actual[16];ok(g_pd3dDevice->GetVertexShaderConstantF(0,actual,4)==S_OK&&actual[3]==matrix[3]);
  ok(g_pd3dDevice->Clear(0,NULL,
 #ifdef DEPTH
 3,0xff0000ff,frame?1.f:0.f,
 #else
 1,0xff0000ff,1,
 #endif
 0)==S_OK);
 #ifdef TRANSFORMED
 DWORD viewport[6]={frame?8u:0u,frame?4u:0u,frame?16u:32u,frame?20u:32u,0,0x3f800000};
 ok(g_pd3dDevice->SetViewport(viewport)==S_OK);
 DWORD observed[8]={0x719bad15,0,0,0,0,0,0,0x719bad15};
 BOOL same=g_pd3dDevice->GetViewport(observed+1)==S_OK&&observed[0]==0x719bad15&&observed[7]==0x719bad15;
 for(UINT n=0;n<6;n++)if(observed[n+1]!=viewport[n])same=FALSE;ok(same);
 DWORD invalid[6]={31,0,2,32,0,0x3f800000};
 ok(g_pd3dDevice->SetViewport(invalid)==(HRESULT)0x8876086c);
 same=g_pd3dDevice->GetViewport(observed+1)==S_OK;
 for(UINT n=0;n<6;n++)if(observed[n+1]!=viewport[n])same=FALSE;ok(same);
 #endif
 ok(g_pd3dDevice->BeginScene()==S_OK);ok(
 #ifdef TRANSFORMED
 g_pd3dDevice->DrawPrimitive(5,0,2)
 #elif defined(INDEXED)
 g_pd3dDevice->DrawIndexedPrimitive(4,-1,2,3,1,1)
 #else
 g_pd3dDevice->DrawPrimitive(4,0,1)
 #endif
 ==S_OK);ok(g_pd3dDevice->EndScene()==S_OK);
  ok(g_pd3dDevice->GetRenderTargetData(back,copy)==S_OK);
  D3DLOCKED_RECT lock={};ok(copy->LockRect(&lock,NULL,D3DLOCK_READONLY)==S_OK);if(!lock.pBits)return 11;
  for(UINT y=0;y<32;y++)for(UINT x=0;x<32;x++)put(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]&0xffffff);
  ok(copy->UnlockRect()==S_OK);
 }
 copy->Release();back->Release();
 #ifdef TEXTURES
 if(g_pd3dDevice->SetTexture(0,NULL)!=S_OK)return 15;
 #endif
 put(0xfeedffff);
 ok(g_pd3dDevice->SetVertexShader(NULL)==S_OK);ok(g_pd3dDevice->SetPixelShader(NULL)==S_OK);ok(g_pd3dDevice->SetVertexDeclaration(NULL)==S_OK);ok(g_pd3dDevice->SetStreamSource(0,NULL,0,0)==S_OK);
 Cleanup();ok(DestroyWindow(window));ok(UnregisterClassW(wc.lpszClassName,wc.hInstance));ok(stack()==before);put(used+1);
 DWORD done=0;if(!WriteFile(GetStdHandle((DWORD)-11),words,used*4,&done,NULL)||done!=used*4)return 12;return 0;
}
