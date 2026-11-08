/* Original MIT observer: the real DLL initializer needs its own source path,
 * before the EXE runs. No manifest/system-directory substitution is involved. */
typedef unsigned int U32;
typedef unsigned short W;
typedef void *H;
__declspec(dllimport) U32 __stdcall GetModuleFileNameW(H,W *,U32);
static H self;
static W name[2048];
int __stdcall DllMain(H module,U32 reason,H reserved) {
    (void)reserved;
    if(reason==1) {
        U32 n=GetModuleFileNameW(module,name,2048);
        const char tail[]="module-path.dll";
        if(n<15||n>=2048||name[n]!=0)return 0;
        for(U32 i=0;i<15;i++)if(name[n-15+i]!=(W)tail[i])return 0;
        self=module;
    }
    return 1;
}
__declspec(dllexport) H __stdcall SourceModule(void) { return self; }
