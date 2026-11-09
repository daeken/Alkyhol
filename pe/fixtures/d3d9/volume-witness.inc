// Owned MIT volume ABI/storage witness. No original game code or assets.
static IDirect3DVolumeTexture9* volume_texture;
static IDirect3DVolumeTexture9* volume_luminance;
static DWORD volume_value(UINT mip,UINT z){return mip?0xff0000ffu:(z<2?0xffff0000u:0xff00ff00u);}
static DWORD volume_witness(){
 #define REQUIRE_VOLUME(test,stage) if(!(test))return stage
 REQUIRE_VOLUME(g_pD3D->CheckDeviceFormat(0,1,22,0,4,21)==S_OK,1);
 REQUIRE_VOLUME(g_pd3dDevice->CreateVolumeTexture(4,2,4,0,0,21,1,&volume_texture,NULL)==S_OK&&volume_texture,2);
 REQUIRE_VOLUME(volume_texture->GetType()==4&&volume_texture->GetLevelCount()==3,3);
 const GUID volume_iid={0x2518526c,0xe789,0x4111,{0xa7,0xb9,0x47,0xef,0x32,0x8d,0x13,0xe6}};
 const GUID level_iid={0x24f416e6,0x1f67,0x4aa7,{0xb8,0x8e,0xd3,0x3f,0x6f,0x31,0x28,0xa1}};
 void* identity=NULL;REQUIRE_VOLUME(volume_texture->QueryInterface(volume_iid,&identity)==S_OK&&identity==volume_texture,4);((IDirect3DVolumeTexture9*)identity)->Release();
 for(UINT mip=0;mip<3;mip++){
  UINT w=4>>mip,h=mip?1:2,d=4>>mip;D3DVOLUME_DESC desc={};
  REQUIRE_VOLUME(volume_texture->GetLevelDesc(mip,&desc)==S_OK&&desc.Format==21&&desc.Type==2&&desc.Pool==1&&desc.Width==w&&desc.Height==h&&desc.Depth==d,5);
  D3DLOCKED_BOX lock={};REQUIRE_VOLUME(volume_texture->LockBox(mip,&lock,NULL,0)==S_OK&&lock.pBits&&lock.RowPitch>=(INT)(w*4)&&lock.SlicePitch>=lock.RowPitch*(INT)h,6);
  for(UINT z=0;z<d;z++)for(UINT y=0;y<h;y++)for(UINT x=0;x<w;x++)((DWORD*)((unsigned char*)lock.pBits+z*lock.SlicePitch+y*lock.RowPitch))[x]=volume_value(mip,z);
  REQUIRE_VOLUME(volume_texture->UnlockBox(mip)==S_OK,7);
  IDirect3DVolume9 *level=NULL,*again=NULL;REQUIRE_VOLUME(volume_texture->GetVolumeLevel(mip,&level)==S_OK&&level&&volume_texture->GetVolumeLevel(mip,&again)==S_OK&&again==level,8);again->Release();
  identity=NULL;REQUIRE_VOLUME(level->QueryInterface(level_iid,&identity)==S_OK&&identity==level,9);((IDirect3DVolume9*)identity)->Release();
  IDirect3DDevice9* parent=NULL;REQUIRE_VOLUME(level->GetDevice(&parent)==S_OK&&parent==g_pd3dDevice,10);parent->Release();
  identity=NULL;REQUIRE_VOLUME(level->GetContainer(volume_iid,&identity)==S_OK&&identity==volume_texture,11);((IDirect3DVolumeTexture9*)identity)->Release();
  REQUIRE_VOLUME(level->GetDesc(&desc)==S_OK&&desc.Width==w&&desc.Height==h&&desc.Depth==d,12);
  REQUIRE_VOLUME(level->LockBox(&lock,NULL,16)==S_OK&&lock.pBits,13);
  for(UINT z=0;z<d;z++)for(UINT y=0;y<h;y++)for(UINT x=0;x<w;x++)REQUIRE_VOLUME(((DWORD*)((unsigned char*)lock.pBits+z*lock.SlicePitch+y*lock.RowPitch))[x]==volume_value(mip,z),14);
  REQUIRE_VOLUME(level->UnlockBox()==S_OK,15);level->Release();
 }
 D3DBOX region={1,1,2,2,1,2};D3DLOCKED_BOX lock={};
 REQUIRE_VOLUME(volume_texture->LockBox(0,&lock,&region,0)==S_OK&&lock.pBits,16);*(DWORD*)lock.pBits=0xff123456;
 REQUIRE_VOLUME(volume_texture->UnlockBox(0)==S_OK,17);
 REQUIRE_VOLUME(volume_texture->LockBox(0,&lock,NULL,16)==S_OK&&*(DWORD*)((unsigned char*)lock.pBits+lock.SlicePitch+lock.RowPitch+4)==0xff123456,18);
 REQUIRE_VOLUME(volume_texture->UnlockBox(0)==S_OK,19);
 IDirect3DVolume9* retained=NULL;REQUIRE_VOLUME(volume_texture->GetVolumeLevel(0,&retained)==S_OK&&retained,20);
 REQUIRE_VOLUME(g_pd3dDevice->SetTexture(0,volume_texture)==S_OK,21);volume_texture->Release();
 IDirect3DBaseTexture9* texture=NULL;REQUIRE_VOLUME(g_pd3dDevice->GetTexture(0,&texture)==S_OK&&texture==volume_texture&&texture->GetType()==4,22);texture->Release();
 identity=NULL;REQUIRE_VOLUME(retained->GetContainer(volume_iid,&identity)==S_OK&&identity==volume_texture,23);((IDirect3DVolumeTexture9*)identity)->Release();retained->Release();
 for(UINT axis=1;axis<=3;axis++)REQUIRE_VOLUME(g_pd3dDevice->SetSamplerState(0,axis,3)==S_OK,24);
 REQUIRE_VOLUME(g_pd3dDevice->CreateVolumeTexture(2,2,2,0,0,50,1,&volume_luminance,NULL)==S_OK&&volume_luminance&&volume_luminance->GetLevelCount()==2,25);
 for(UINT mip=0;mip<2;mip++){
  D3DVOLUME_DESC desc={};REQUIRE_VOLUME(volume_luminance->GetLevelDesc(mip,&desc)==S_OK&&desc.Format==50&&desc.Depth==(mip?1u:2u),26);
  REQUIRE_VOLUME(volume_luminance->LockBox(mip,&lock,NULL,0)==S_OK&&lock.pBits&&lock.RowPitch>=(INT)desc.Width&&lock.SlicePitch>=lock.RowPitch*(INT)desc.Height,27);
  for(UINT z=0;z<desc.Depth;z++)for(UINT y=0;y<desc.Height;y++)for(UINT x=0;x<desc.Width;x++)((unsigned char*)lock.pBits)[z*lock.SlicePitch+y*lock.RowPitch+x]=(unsigned char)(mip?128:z?192:64);
  REQUIRE_VOLUME(volume_luminance->UnlockBox(mip)==S_OK&&volume_luminance->LockBox(mip,&lock,NULL,16)==S_OK,28);
  for(UINT z=0;z<desc.Depth;z++)for(UINT y=0;y<desc.Height;y++)for(UINT x=0;x<desc.Width;x++)REQUIRE_VOLUME(((unsigned char*)lock.pBits)[z*lock.SlicePitch+y*lock.RowPitch+x]==(mip?128:z?192:64),29);
  REQUIRE_VOLUME(volume_luminance->UnlockBox(mip)==S_OK,30);
 }
 #undef REQUIRE_VOLUME
 return 0;
}
