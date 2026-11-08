// Original MIT SDK/GDI observer; only owned resource fixture data enters CI.
#define UNICODE
#define _UNICODE
#include <windows.h>
#include <cstdio>
static_assert(sizeof(DWORD)==4&&sizeof(BITMAPINFOHEADER)==40,"SDK icon widths");
int wmain(int argc,wchar_t **argv) {
    if(argc!=3)return 2;
    HMODULE module=LoadLibraryExW(argv[1],nullptr,LOAD_LIBRARY_AS_DATAFILE|LOAD_LIBRARY_AS_IMAGE_RESOURCE);
    if(!module)return 3;
    HICON icon=LoadIconW(module,MAKEINTRESOURCEW(41));ICONINFO ii{};BITMAP color{},mask{};
    if(!icon||!GetIconInfo(icon,&ii)||!ii.fIcon||ii.xHotspot!=16||ii.yHotspot!=16)return 4;
    if(GetObjectW(ii.hbmColor,static_cast<int>(sizeof(color)),&color)!=static_cast<int>(sizeof(color))||GetObjectW(ii.hbmMask,static_cast<int>(sizeof(mask)),&mask)!=static_cast<int>(sizeof(mask)))return 5;
    if(color.bmWidth!=32||color.bmHeight!=32||color.bmBitsPixel!=32||mask.bmWidth!=32||mask.bmHeight!=32||mask.bmBitsPixel!=1)return 6;
    BYTE pixels[4096]{},bits[128]{};HDC dc=GetDC(nullptr);if(!dc)return 7;
    BITMAPINFO ci{};ci.bmiHeader.biSize=sizeof(BITMAPINFOHEADER);ci.bmiHeader.biWidth=32;ci.bmiHeader.biHeight=-32;ci.bmiHeader.biPlanes=1;ci.bmiHeader.biBitCount=32;ci.bmiHeader.biCompression=BI_RGB;
    struct {BITMAPINFOHEADER header;RGBQUAD colors[2];} mi{};
    mi.header.biSize=sizeof(BITMAPINFOHEADER);mi.header.biWidth=32;mi.header.biHeight=-32;mi.header.biPlanes=1;mi.header.biBitCount=1;mi.header.biCompression=BI_RGB;mi.header.biClrUsed=2;
    mi.colors[1].rgbBlue=mi.colors[1].rgbGreen=mi.colors[1].rgbRed=255;
    if(GetDIBits(dc,ii.hbmColor,0,32,pixels,&ci,DIB_RGB_COLORS)!=32||GetDIBits(dc,ii.hbmMask,0,32,bits,reinterpret_cast<BITMAPINFO *>(&mi),DIB_RGB_COLORS)!=32)return 8;
    FILE *out=nullptr;if(_wfopen_s(&out,argv[2],L"wb")||!out)return 9;
    DWORD header[]={0x4149434f,1,32,32,static_cast<DWORD>(sizeof(pixels)),static_cast<DWORD>(sizeof(bits))};
    bool good=fwrite(header,sizeof(header),1,out)==1&&fwrite(pixels,sizeof(pixels),1,out)==1&&fwrite(bits,sizeof(bits),1,out)==1;
    if(fclose(out))good=false;
    ReleaseDC(nullptr,dc);DeleteObject(ii.hbmColor);DeleteObject(ii.hbmMask);FreeLibrary(module);
    return good?0:10;
}
