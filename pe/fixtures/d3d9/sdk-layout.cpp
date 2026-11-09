// Original MIT independent Windows SDK check; never includes Alkyhol headers.
#define CINTERFACE
#include <Windows.h>
#include <d3d9.h>
#include <cstddef>
static_assert(sizeof(void*)==4);
static_assert(sizeof(D3DPRESENT_PARAMETERS)==56);
static_assert(offsetof(D3DPRESENT_PARAMETERS,hDeviceWindow)==28);
static_assert(sizeof(D3DADAPTER_IDENTIFIER9)==1104);
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
