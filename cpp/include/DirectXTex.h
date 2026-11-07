// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include_next <DirectXTex.h>
#include <functional>
struct IWICMetadataQueryReader;
namespace DirectX {
HRESULT __cdecl LoadFromWICFile(const wchar_t*,WIC_FLAGS,TexMetadata*,ScratchImage&,
    std::function<void(IWICMetadataQueryReader*)> getMQR=nullptr);
}
namespace DirectX {
HRESULT __cdecl FlipRotate(const Image&,TEX_FR_FLAGS,ScratchImage&) noexcept;
HRESULT __cdecl FlipRotate(const Image*,size_t,const TexMetadata&,TEX_FR_FLAGS,ScratchImage&) noexcept;
}
