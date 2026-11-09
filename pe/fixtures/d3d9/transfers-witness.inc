// Original MIT staging/transfer witness; every sampled value comes from GPU readback.
static const DWORD transfer_texture_ps[]={
#include "legacy-texture.ps.inc"
};
static const DWORD transfer_cube_ps[]={
#include "legacy-cube.ps.inc"
};
static const DWORD transfer_volume_ps[]={
#include "legacy-volume.ps.inc"
};
static IDirect3DSurface9 *transfer_target=NULL,*transfer_copy=NULL;
static DWORD transfer_sample(IDirect3DBaseTexture9* texture,IDirect3DPixelShader9* shader,float x,float y,float z,UINT mip){
 float v[6][8]={{-1,1,.5f,1,x,y,z,1},{1,1,.5f,1,x,y,z,1},{-1,-1,.5f,1,x,y,z,1},{-1,-1,.5f,1,x,y,z,1},{1,1,.5f,1,x,y,z,1},{1,-1,.5f,1,x,y,z,1}};
 if(g_pd3dDevice->SetRenderTarget(0,transfer_target)!=S_OK||g_pd3dDevice->SetPixelShader(shader)!=S_OK||g_pd3dDevice->SetTexture(0,texture)!=S_OK)return 0xdeadc001;
 if(g_pd3dDevice->SetSamplerState(0,9,mip)!=S_OK||g_pd3dDevice->Clear(0,NULL,1,0xff123456,1,0)!=S_OK||g_pd3dDevice->BeginScene()!=S_OK)return 0xdeadc002;
 if(g_pd3dDevice->DrawPrimitiveUP(4,2,v,32)!=S_OK||g_pd3dDevice->EndScene()!=S_OK||g_pd3dDevice->GetRenderTargetData(transfer_target,transfer_copy)!=S_OK)return 0xdeadc003;
 D3DLOCKED_RECT lock={};if(transfer_copy->LockRect(&lock,NULL,16)!=S_OK||!lock.pBits)return 0xdeadc004;
 DWORD value=*(DWORD*)lock.pBits;if(transfer_copy->UnlockRect()!=S_OK)return 0xdeadc005;return value;
}
static DWORD transfers_witness(){
 #define REQUIRE_TRANSFER(test,stage) if(!(test))return stage
 IDirect3DSurface9 *original=NULL,*rt=NULL,*staging=NULL,*copy=NULL;
 IDirect3DVertexBuffer9* stream=NULL;UINT offset=0,stride=0;IDirect3DPixelShader9* original_ps=NULL;
 REQUIRE_TRANSFER(g_pd3dDevice->GetRenderTarget(0,&original)==S_OK&&original&&g_pd3dDevice->GetStreamSource(0,&stream,&offset,&stride)==S_OK&&stream&&g_pd3dDevice->GetPixelShader(&original_ps)==S_OK&&original_ps,1);
 REQUIRE_TRANSFER(g_pd3dDevice->CreateRenderTarget(6,5,21,0,0,FALSE,&rt,NULL)==S_OK&&rt&&g_pd3dDevice->CreateOffscreenPlainSurface(6,5,21,2,&copy,NULL)==S_OK&&copy&&g_pd3dDevice->CreateOffscreenPlainSurface(8,4,21,2,&staging,NULL)==S_OK&&staging,2);
 D3DSURFACE_DESC desc={};REQUIRE_TRANSFER(rt->GetDesc(&desc)==S_OK&&desc.Width==6&&desc.Height==5&&desc.Format==21&&desc.Usage==1&&desc.Pool==0,3);
 // ColorFill ignores color write masks and viewport draw state.
 REQUIRE_TRANSFER(g_pd3dDevice->SetRenderState(168,0)==S_OK&&g_pd3dDevice->SetRenderState(194,1)==S_OK&&g_pd3dDevice->ColorFill(rt,NULL,0x80203040)==S_OK,4);
 int rect[4]={2,1,5,4};REQUIRE_TRANSFER(g_pd3dDevice->ColorFill(rt,rect,0x507090b0)==S_OK&&g_pd3dDevice->GetRenderTargetData(rt,copy)==S_OK,5);
 D3DLOCKED_RECT lock={};REQUIRE_TRANSFER(copy->LockRect(&lock,NULL,16)==S_OK&&lock.pBits,6);
 for(UINT y=0;y<5;y++)for(UINT x=0;x<6;x++)REQUIRE_TRANSFER(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]==(x>=2&&x<5&&y>=1&&y<4?0x507090b0u:0x80203040u),7);
 REQUIRE_TRANSFER(copy->UnlockRect()==S_OK&&g_pd3dDevice->SetRenderState(168,15)==S_OK&&g_pd3dDevice->SetRenderState(194,0)==S_OK,8);
 REQUIRE_TRANSFER(staging->LockRect(&lock,NULL,0)==S_OK&&lock.pBits,9);
 for(UINT y=0;y<4;y++)for(UINT x=0;x<8;x++)((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]=0xff000000|(y<<16)|(x<<8)|0x55;
 REQUIRE_TRANSFER(FAILED(g_pd3dDevice->UpdateSurface(staging,NULL,rt,NULL))&&staging->UnlockRect()==S_OK,10);
 int sr[4]={1,1,4,3},point[2]={2,3};REQUIRE_TRANSFER(g_pd3dDevice->UpdateSurface(staging,sr,rt,point)==S_OK&&g_pd3dDevice->GetRenderTargetData(rt,copy)==S_OK,11);
 REQUIRE_TRANSFER(copy->LockRect(&lock,NULL,16)==S_OK&&lock.pBits,12);
 for(UINT y=0;y<5;y++)for(UINT x=0;x<6;x++){
  DWORD expected=x>=2&&x<5&&y>=1&&y<4?0x507090b0:0x80203040;
  if(x>=2&&x<5&&y>=3)expected=0xff000000|((y-2)<<16)|((x-1)<<8)|0x55;
  REQUIRE_TRANSFER(((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]==expected,13);
 }
 REQUIRE_TRANSFER(copy->UnlockRect()==S_OK,14);
 int bad[2]={5,4};REQUIRE_TRANSFER(FAILED(g_pd3dDevice->UpdateSurface(staging,sr,rt,bad))&&FAILED(g_pd3dDevice->ColorFill(staging,NULL,0)),15);
 REQUIRE_TRANSFER(staging->LockRect(&lock,sr,0)==S_OK&&lock.Pitch>=32&&lock.pBits,16);*(DWORD*)lock.pBits=0xffaa55cc;
 REQUIRE_TRANSFER(staging->UnlockRect()==S_OK&&g_pd3dDevice->UpdateSurface(staging,sr,rt,point)==S_OK,17);
 REQUIRE_TRANSFER(g_pd3dDevice->CreateRenderTarget(1,1,21,0,0,FALSE,&transfer_target,NULL)==S_OK&&g_pd3dDevice->CreateOffscreenPlainSurface(1,1,21,2,&transfer_copy,NULL)==S_OK,18);
 IDirect3DPixelShader9 *ps2=NULL,*psc=NULL,*psv=NULL;
 REQUIRE_TRANSFER(g_pd3dDevice->CreatePixelShader(transfer_texture_ps,&ps2)==S_OK&&g_pd3dDevice->CreatePixelShader(transfer_cube_ps,&psc)==S_OK&&g_pd3dDevice->CreatePixelShader(transfer_volume_ps,&psv)==S_OK,19);
 REQUIRE_TRANSFER(g_pd3dDevice->SetVertexShaderConstantF(0,matrix,4)==S_OK,20);
 for(UINT state=1;state<=3;state++)REQUIRE_TRANSFER(g_pd3dDevice->SetSamplerState(0,state,3)==S_OK,21);
 REQUIRE_TRANSFER(g_pd3dDevice->SetSamplerState(0,7,1)==S_OK,22);
 IDirect3DTexture9 *source=NULL,*destination=NULL,*tail=NULL;
 REQUIRE_TRANSFER(g_pd3dDevice->CreateTexture(4,4,0,0,21,2,&source,NULL)==S_OK&&g_pd3dDevice->CreateTexture(4,4,0,0,21,0,&destination,NULL)==S_OK&&g_pd3dDevice->CreateTexture(2,2,0,0,21,0,&tail,NULL)==S_OK,23);
 for(UINT mip=0;mip<3;mip++){
  REQUIRE_TRANSFER(source->LockRect(mip,&lock,NULL,0)==S_OK&&lock.pBits,24);
  for(UINT y=0;y<(4u>>mip);y++)for(UINT x=0;x<(4u>>mip);x++)((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]=0xff203040+mip*0x00101010;
  REQUIRE_TRANSFER(source->UnlockRect(mip)==S_OK,25);
 }
 REQUIRE_TRANSFER(FAILED(g_pd3dDevice->SetTexture(0,source))&&g_pd3dDevice->UpdateTexture(source,destination)==S_OK,26);
 for(UINT mip=0;mip<3;mip++)REQUIRE_TRANSFER(transfer_sample(destination,ps2,.25f,.25f,0,mip)==0xff203040+mip*0x00101010,27);
 REQUIRE_TRANSFER(source->AddDirtyRect(NULL)==S_OK&&g_pd3dDevice->UpdateTexture(source,tail)==S_OK,28);
 for(UINT mip=0;mip<2;mip++)REQUIRE_TRANSFER(transfer_sample(tail,ps2,.25f,.25f,0,mip)==0xff304050+mip*0x00101010,29);
 REQUIRE_TRANSFER(source->LockRect(0,&lock,NULL,0x8000)==S_OK&&lock.pBits,30);
 for(UINT y=0;y<4;y++)for(UINT x=0;x<4;x++)((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]=0xff998877;
 REQUIRE_TRANSFER(source->UnlockRect(0)==S_OK&&g_pd3dDevice->UpdateTexture(source,destination)==S_OK&&transfer_sample(destination,ps2,.25f,.25f,0,0)==0xff203040,31);
 REQUIRE_TRANSFER(source->AddDirtyRect(NULL)==S_OK&&g_pd3dDevice->UpdateTexture(source,destination)==S_OK&&transfer_sample(destination,ps2,.25f,.25f,0,0)==0xff998877,32);
 IDirect3DSurface9* destination_surface=NULL;REQUIRE_TRANSFER(destination->GetSurfaceLevel(0,&destination_surface)==S_OK&&destination_surface,33);
 int one[4]={1,1,2,2},origin[2]={0,0};REQUIRE_TRANSFER(g_pd3dDevice->UpdateSurface(staging,one,destination_surface,origin)==S_OK&&transfer_sample(destination,ps2,.125f,.125f,0,0)==0xffaa55cc,34);destination_surface->Release();
 IDirect3DCubeTexture9 *cube_source=NULL,*cube_dest=NULL;
 REQUIRE_TRANSFER(g_pd3dDevice->CreateCubeTexture(2,0,0,21,2,&cube_source,NULL)==S_OK&&g_pd3dDevice->CreateCubeTexture(2,0,0,21,0,&cube_dest,NULL)==S_OK,35);
 for(UINT face=0;face<6;face++)for(UINT mip=0;mip<2;mip++){
  REQUIRE_TRANSFER(cube_source->LockRect(face,mip,&lock,NULL,0)==S_OK&&lock.pBits,36);
  for(UINT y=0;y<(2u>>mip);y++)for(UINT x=0;x<(2u>>mip);x++)((DWORD*)((unsigned char*)lock.pBits+y*lock.Pitch))[x]=0xff102030+face*0x000a0000+mip*0x00000a00;
  REQUIRE_TRANSFER(cube_source->UnlockRect(face,mip)==S_OK,37);
 }
 REQUIRE_TRANSFER(g_pd3dDevice->UpdateTexture(cube_source,cube_dest)==S_OK,38);
 const float axes[6][3]={{1,0,0},{-1,0,0},{0,1,0},{0,-1,0},{0,0,1},{0,0,-1}};
 for(UINT face=0;face<6;face++)for(UINT mip=0;mip<2;mip++)REQUIRE_TRANSFER(transfer_sample(cube_dest,psc,axes[face][0],axes[face][1],axes[face][2],mip)==0xff102030+face*0x000a0000+mip*0x00000a00,39);
 REQUIRE_TRANSFER(cube_source->AddDirtyRect(3,NULL)==S_OK&&g_pd3dDevice->UpdateTexture(cube_source,cube_dest)==S_OK,40);
 for(UINT format=21;format<=50;format+=29){
  IDirect3DVolumeTexture9 *volume_source=NULL,*volume_dest=NULL;
  REQUIRE_TRANSFER(g_pd3dDevice->CreateVolumeTexture(2,2,2,0,0,format,2,&volume_source,NULL)==S_OK&&g_pd3dDevice->CreateVolumeTexture(2,2,2,0,0,format,0,&volume_dest,NULL)==S_OK,41);
  for(UINT mip=0;mip<2;mip++){
   D3DLOCKED_BOX box={};REQUIRE_TRANSFER(volume_source->LockBox(mip,&box,NULL,0)==S_OK&&box.pBits,42);
   for(UINT z=0;z<(2u>>mip);z++)for(UINT y=0;y<(2u>>mip);y++)for(UINT x=0;x<(2u>>mip);x++){
    unsigned char* p=(unsigned char*)box.pBits+z*box.SlicePitch+y*box.RowPitch;
    if(format==21)((DWORD*)p)[x]=0xff204060+z*0x00200000+mip*0x00002000;else p[x]=(unsigned char)(64+z*64+mip*32);
   }
   REQUIRE_TRANSFER(volume_source->UnlockBox(mip)==S_OK,43);
  }
  REQUIRE_TRANSFER(g_pd3dDevice->UpdateTexture(volume_source,volume_dest)==S_OK,44);
  for(UINT slice=0;slice<2;slice++)REQUIRE_TRANSFER(transfer_sample(volume_dest,psv,.25f,.25f,.25f+slice*.5f,0)==(format==21?0xff204060+slice*0x00200000:0xff404040+slice*0x00404040),45);
  REQUIRE_TRANSFER(transfer_sample(volume_dest,psv,.25f,.25f,.25f,1)==(format==21?0xff206060u:0xff606060u),46);
  REQUIRE_TRANSFER(volume_source->AddDirtyBox(NULL)==S_OK&&g_pd3dDevice->UpdateTexture(volume_source,volume_dest)==S_OK,47);
  volume_source->Release();volume_dest->Release();
 }
 REQUIRE_TRANSFER(g_pd3dDevice->SetTexture(0,NULL)==S_OK&&g_pd3dDevice->SetPixelShader(original_ps)==S_OK&&g_pd3dDevice->SetRenderTarget(0,original)==S_OK&&g_pd3dDevice->SetStreamSource(0,stream,offset,stride)==S_OK&&g_pd3dDevice->SetSamplerState(0,9,0)==S_OK,48);
 source->Release();destination->Release();tail->Release();cube_source->Release();cube_dest->Release();
 original->Release();original_ps->Release();stream->Release();ps2->Release();psc->Release();psv->Release();rt->Release();staging->Release();copy->Release();transfer_target->Release();transfer_copy->Release();
 #undef REQUIRE_TRANSFER
 return 0;
}
