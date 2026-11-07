// Copyright (c) Alkyhol contributors. MIT licensed.
#include <Windows.h>
#include <dinput.h>
#include <cassert>
#include <cstdio>
#include <initializer_list>
int main(){
    // Public SDK omits this wire-bit spelling; pinned MinGW names it DIDFT_OPTIONAL.
    constexpr DWORD optional=0x80000000u;
    assert(c_dfDIKeyboard.dwSize==sizeof(DIDATAFORMAT)&&c_dfDIKeyboard.dwObjSize==sizeof(DIOBJECTDATAFORMAT));
    assert(c_dfDIKeyboard.dwDataSize==256&&c_dfDIKeyboard.dwNumObjs==256&&c_dfDIKeyboard.dwFlags==DIDF_RELAXIS);
    for(unsigned i=0;i<256;++i){const auto& o=c_dfDIKeyboard.rgodf[i];assert(o.pguid&&*o.pguid==GUID_Key&&o.dwOfs==i&&o.dwType==(DIDFT_BUTTON|DIDFT_MAKEINSTANCE(i)|optional)&&o.dwFlags==0);}
    assert(c_dfDIMouse2.dwSize==sizeof(DIDATAFORMAT)&&c_dfDIMouse2.dwObjSize==sizeof(DIOBJECTDATAFORMAT));
    assert(c_dfDIMouse2.dwDataSize==20&&c_dfDIMouse2.dwNumObjs==11&&c_dfDIMouse2.dwFlags==DIDF_RELAXIS);
    const GUID* axes[]={&GUID_XAxis,&GUID_YAxis,&GUID_ZAxis};
    for(unsigned i=0;i<11;++i){const auto& o=c_dfDIMouse2.rgodf[i];assert(o.dwFlags==0);if(i<3){assert(o.pguid&&*o.pguid==*axes[i]&&o.dwOfs==4*i);assert(o.dwType==(DIDFT_AXIS|DIDFT_ANYINSTANCE|(i==2?optional:0)));}else{assert(!o.pguid&&o.dwOfs==i+9&&o.dwType==(DIDFT_BUTTON|DIDFT_ANYINSTANCE|(i>=5?optional:0)));}}
    for(const auto* format:{&c_dfDIKeyboard,&c_dfDIMouse2}){
        std::printf("format %u %u %u %u %u\n",format->dwSize,format->dwObjSize,format->dwFlags,format->dwDataSize,format->dwNumObjs);
        for(unsigned i=0;i<format->dwNumObjs;++i){const auto& o=format->rgodf[i];std::printf("%u %u %u ",o.dwOfs,o.dwType,o.dwFlags);if(o.pguid){const auto* bytes=reinterpret_cast<const unsigned char*>(o.pguid);for(unsigned j=0;j<sizeof(GUID);++j)std::printf("%02x",unsigned(bytes[j]));}else std::printf("null");std::puts("");}
    }
    std::puts("DirectInput metadata: 256 keyboard keys and 11 mouse objects passed (device support separate)");
}
