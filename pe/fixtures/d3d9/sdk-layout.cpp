// Original MIT independent Windows SDK check; never includes Alkyhol headers.
#define CINTERFACE
#include <Windows.h>
#include <d3d9.h>
#include <cstddef>
static_assert(sizeof(void*)==4);
static_assert(sizeof(D3DPRESENT_PARAMETERS)==56);
static_assert(offsetof(D3DPRESENT_PARAMETERS,hDeviceWindow)==28);
static_assert(sizeof(D3DADAPTER_IDENTIFIER9)==1100);
static_assert(offsetof(D3DADAPTER_IDENTIFIER9,VendorId)==1064);
static_assert(offsetof(D3DADAPTER_IDENTIFIER9,DeviceIdentifier)==1080);
static_assert(sizeof(D3DSURFACE_DESC)==32);
static_assert(sizeof(D3DLOCKED_RECT)==8);
static_assert(offsetof(IDirect3D9Vtbl,CreateDevice)==16*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,Present)==17*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,GetBackBuffer)==18*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,GetRenderTargetData)==32*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,CreateOffscreenPlainSurface)==36*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,BeginScene)==41*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,Clear)==43*4);
static_assert(offsetof(IDirect3DSurface9Vtbl,LockRect)==13*4);

static_assert(sizeof(D3DVERTEXBUFFER_DESC)==24);
static_assert(offsetof(IDirect3DDevice9Vtbl,CreateVertexBuffer)==26*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,DrawPrimitive)==81*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,SetFVF)==89*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,SetStreamSource)==100*4);
static_assert(offsetof(IDirect3DVertexBuffer9Vtbl,Lock)==11*4);
static_assert(offsetof(IDirect3DVertexBuffer9Vtbl,Unlock)==12*4);

static_assert(sizeof(D3DDISPLAYMODE)==16);
static_assert(sizeof(D3DCAPS9)==304);
static_assert(offsetof(D3DCAPS9,MaxPrimitiveCount)==180);
static_assert(offsetof(D3DCAPS9,MaxStreams)==188);
static_assert(offsetof(D3DCAPS9,MaxStreamStride)==192);
static_assert(offsetof(D3DCAPS9,VertexShaderVersion)==196);
static_assert(offsetof(D3DCAPS9,PixelShaderVersion)==204);
static_assert(offsetof(D3DCAPS9,NumberOfAdaptersInGroup)==232);
static_assert(offsetof(D3DCAPS9,NumSimultaneousRTs)==240);

static_assert(sizeof(D3DVERTEXELEMENT9)==8);
static_assert(offsetof(IDirect3DDevice9Vtbl,CreateVertexDeclaration)==86*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,SetVertexDeclaration)==87*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,CreateVertexShader)==91*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,SetVertexShaderConstantF)==94*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,CreatePixelShader)==106*4);
static_assert(offsetof(IDirect3DDevice9Vtbl,SetPixelShaderConstantF)==109*4);
static_assert(offsetof(IDirect3DVertexShader9Vtbl,GetFunction)==4*4);
static_assert(offsetof(IDirect3DPixelShader9Vtbl,GetFunction)==4*4);
static_assert(offsetof(IDirect3DVertexDeclaration9Vtbl,GetDeclaration)==4*4);
