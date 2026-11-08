/* Native Windows i386 loader traverses the actual unchanged DLL dependency. */
typedef unsigned int U32;
__declspec(dllimport) void *__stdcall LoadLibraryA(const char *);
__declspec(dllimport) void *__stdcall GetProcAddress(void *,const char *);
__declspec(dllimport) int __stdcall FreeLibrary(void *);
typedef U32 (__cdecl *Run)(U32,U32,U32);
U32 entry(void) {
    void *lib=LoadLibraryA("consumer.dll");
    if(!lib)return 10;
    Run run=(Run)GetProcAddress(lib,"Run");
    if(!run)return 11;
    U32 result=run(0xabcdef01,0x12345678,19);
    if(result!=(0xb9f9b98c^0xeeddcca5^0x2468ace0))return 12;
    return FreeLibrary(lib)?0:13;
}
