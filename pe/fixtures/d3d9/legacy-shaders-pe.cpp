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
static const float vertices[][8]={{0,.75f,.5f,1,1,0,0,1},{.75f,-.75f,.5f,1,1,0,0,1},{-.75f,-.75f,.5f,1,1,0,0,1}};
static float matrix[16]={1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1};
static const D3DVERTEXELEMENT9 elements[]={{0,0,3,0,0,0},{0,16,3,0,10,0},{255,0,17,0,0,0}};
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
extern "C" DWORD entry(){
 DWORD before=stack();put(0x39534844);
#ifdef TARGETS
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
 #ifdef MODEL2
 g_pD3D=Direct3DCreate9(32);if(!g_pD3D)return 4;
 D3DPRESENT_PARAMETERS present={};present.BackBufferWidth=32;present.BackBufferHeight=32;present.BackBufferFormat=21;present.BackBufferCount=1;present.SwapEffect=1;present.hDeviceWindow=window;present.Windowed=TRUE;present.PresentationInterval=0x80000000;
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
 D3DVERTEXBUFFER_DESC vb_desc={};ok(g_pd3dDevice->CreateVertexBuffer(sizeof(vertices),8,0,0,&vb,NULL)==S_OK&&vb&&vb->GetDesc(&vb_desc)==S_OK&&vb_desc.Usage==8&&vb_desc.Size==sizeof(vertices));if(!vb)return 8;
 void* storage=NULL;ok(vb->Lock(0,sizeof(vertices),&storage,0)==S_OK);if(!storage)return 9;
 for(UINT i=0;i<sizeof(vertices);i++)((unsigned char*)storage)[i]=((const unsigned char*)vertices)[i];ok(vb->Unlock()==S_OK);
 ok(g_pd3dDevice->SetStreamSource(0,vb,0,32)==S_OK);
 // Internal bindings retain resources, including shader bytecode and declaration.
 vs->Release();ps->Release();decl->Release();vb->Release();
 IDirect3DVertexShader9* bound=NULL;ok(g_pd3dDevice->GetVertexShader(&bound)==S_OK&&bound==vs);if(bound)bound->Release();
 IDirect3DSurface9 *back=NULL,*copy=NULL;ok(g_pd3dDevice->GetBackBuffer(0,0,0,&back)==S_OK);ok(g_pd3dDevice->CreateOffscreenPlainSurface(32,32,
#ifdef MODEL2
21,
#else
22,
#endif
2,&copy,NULL)==S_OK);if(!back||!copy)return 10;
 #ifdef TEXTURES
 IDirect3DTexture9* texture=NULL;ok(g_pd3dDevice->CreateTexture(1,1,1,0,21,1,&texture,NULL)==S_OK);if(!texture)return 13;
 ok(texture->GetLevelCount()==1);D3DSURFACE_DESC desc={};ok(texture->GetLevelDesc(0,&desc)==S_OK&&desc.Format==21&&desc.Type==1&&desc.Pool==1&&desc.Width==1&&desc.Height==1);
 INT whole_texture[4]={0,0,1,1};D3DLOCKED_RECT texlock={};ok(texture->LockRect(0,&texlock,whole_texture,0)==S_OK);if(!texlock.pBits)return 14;*(DWORD*)texlock.pBits=0x80808080;
 ok(texture->UnlockRect(0)==S_OK);ok(g_pd3dDevice->SetTexture(0,texture)==S_OK);
 IDirect3DBaseTexture9* retrieved=NULL;ok(g_pd3dDevice->GetTexture(0,&retrieved)==S_OK&&retrieved==texture);if(retrieved)retrieved->Release();
 parent=NULL;ok(texture->GetDevice(&parent)==S_OK&&parent==g_pd3dDevice);if(parent)parent->Release();ok(texture->GetType()==3);
 DWORD state=99;ok(g_pd3dDevice->GetSamplerState(0,11,&state)==S_OK&&state==0);ok(g_pd3dDevice->SetSamplerState(0,5,2)==S_OK);ok(g_pd3dDevice->GetSamplerState(0,5,&state)==S_OK&&state==2);
 ok(g_pD3D->CheckDeviceFormat(0,1,22,0x80000,3,21)==S_OK);
 const GUID base_texture={0x580ca87e,0x1d3c,0x4d54,{0x99,0x1d,0xb7,0xd3,0xe3,0xc2,0x98,0xce}};void* texture_identity=NULL;
 ok(texture->QueryInterface(base_texture,&texture_identity)==S_OK&&texture_identity==texture);if(texture_identity)((IDirect3DBaseTexture9*)texture_identity)->Release();
 texture->Release();
 #ifdef TARGETS
 DWORD target_stage=render_target_witness(back);ok(target_stage==0);if(target_stage)return fail(16,target_stage);
 #endif
 #endif
 #ifdef STATES
 ok(g_pd3dDevice->SetRenderState(27,1)==S_OK);ok(g_pd3dDevice->SetRenderState(19,5)==S_OK);ok(g_pd3dDevice->SetRenderState(20,6)==S_OK);
 DWORD frequency=0;ok(g_pd3dDevice->SetStreamSourceFreq(0,1)==S_OK&&g_pd3dDevice->GetStreamSourceFreq(0,&frequency)==S_OK&&frequency==1);
 IDirect3DIndexBuffer9* index=NULL;ok(g_pd3dDevice->SetIndices(NULL)==S_OK&&g_pd3dDevice->GetIndices(&index)==S_OK&&index==NULL);
 DWORD operation=0;ok(g_pd3dDevice->GetTextureStageState(0,1,&operation)==S_OK&&operation==4&&g_pd3dDevice->SetTextureStageState(0,1,1)==S_OK);
 ok(g_pd3dDevice->GetTextureStageState(0,1,&operation)==S_OK&&operation==1);
 DWORD blend=0;ok(g_pd3dDevice->GetRenderState(7,&blend)==S_OK&&blend==0&&g_pd3dDevice->GetRenderState(27,&blend)==S_OK&&blend==1);
 #endif
 for(UINT frame=0;frame<2;frame++){
  matrix[3]=frame?.75f:0;
  put(0xfeed0000+frame);
  #ifdef STATES
  ok(g_pd3dDevice->SetRenderState(162,frame?0xffffffffu:0)==S_OK);DWORD mask=1;ok(g_pd3dDevice->GetRenderState(162,&mask)==S_OK&&mask==(frame?0xffffffffu:0));
  ok(g_pd3dDevice->SetRenderState(168,frame?1:15)==S_OK);DWORD writes=0;ok(g_pd3dDevice->GetRenderState(168,&writes)==S_OK&&writes==(frame?1u:15u));
  #endif
  #ifdef TEXTURES
  ok(g_pd3dDevice->SetSamplerState(0,11,frame)==S_OK);DWORD srgb=99;ok(g_pd3dDevice->GetSamplerState(0,11,&srgb)==S_OK&&srgb==frame);
  #endif
  ok(g_pd3dDevice->SetVertexShaderConstantF(0,matrix,4)==S_OK);
  static float actual[16];ok(g_pd3dDevice->GetVertexShaderConstantF(0,actual,4)==S_OK&&actual[3]==matrix[3]);
  ok(g_pd3dDevice->Clear(0,NULL,1,0xff0000ff,1,0)==S_OK);ok(g_pd3dDevice->BeginScene()==S_OK);ok(g_pd3dDevice->DrawPrimitive(4,0,1)==S_OK);ok(g_pd3dDevice->EndScene()==S_OK);
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
