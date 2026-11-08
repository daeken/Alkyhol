/* Native Windows observer: executes the identical DLLs supplied to Alkyhol. */
typedef unsigned int U32;
__declspec(dllimport) void *__stdcall LoadLibraryA(const char *);
__declspec(dllimport) void *__stdcall GetProcAddress(void *,const char *);
__declspec(dllimport) int __stdcall FreeLibrary(void *);
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
typedef U32 (__stdcall *Std)(U32,U32,U32,U32,U32);
typedef U32 (__cdecl *Cdecl)(U32,U32,U32);
U32 entry(void) {
    void *lib=LoadLibraryA("accepted.dll");
    if(!lib)return 10;
    Std s=(Std)GetProcAddress(lib,"RunStdcall");
    Cdecl c=(Cdecl)GetProcAddress(lib,"RunCdecl");
    if(!s||!c)return 11;
    U32 a=s(0xfffffff0,7,0x1234,11,0x778899aa);
    U32 b=c(0xabcdef01,0x12345678,19);
    U32 d=s(31,17,3,0xfffffff0,0x11223344);
    if(!FreeLibrary(lib))return 12;
    void *bad=LoadLibraryA("rejected.dll");
    if(bad)return 13;
    U32 words[8]={0x44323349,4,a,b,d,1,1,0xa1b2c3d4},written=0;
    return !WriteFile(GetStdHandle((U32)-11),words,sizeof(words),&written,0)||written!=sizeof(words);
}
