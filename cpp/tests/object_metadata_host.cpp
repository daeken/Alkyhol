// Copyright (c) Alkyhol contributors. MIT licensed.
// Reuse the frozen native method-call sequence verbatim, with an explicitly
// different CPU-only constructor. No device/fence or original application is run.
#define main unused_native_reference_entry
#include "object_private_data.cpp"
#undef main
extern "C" HRESULT AlkyholCreateMetadataObject32(ID3D12Object**);
int main(){
    ComPtr<ID3D12Object> object;
    if(!checked(AlkyholCreateMetadataObject32(&object),"AlkyholCreateMetadataObject32"))return 1;
    ComPtr<IUnknown> identity;if(FAILED(object.As(&identity))||identity.Get()!=object.Get())return 2;identity.Reset();
    std::printf("{\"scenario\":\"d3d12-object-private-data-v1\",\"records\":[");observe(object.Get());object.Reset();event("object.destroyed");std::puts("]}");return 0;
}
