/* Original MIT native observer for the exact CRT names modeled by this gateway.
 * Kernel-only imports let absence be measured before any optional CRT import. */
typedef unsigned int U;typedef void *H;
__declspec(dllimport) H __stdcall LoadLibraryA(const char *);
__declspec(dllimport) H __stdcall GetProcAddress(H,const char *);
__declspec(dllimport) int __stdcall FreeLibrary(H);
__declspec(dllimport) U __stdcall GetVersion(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
U entry(void){
    static const char *names[]={"malloc","_set_error_mode","?set_terminate@@YAP6AXXZP6AXXZ@Z","_get_terminate","?set_unexpected@@YAP6AXXZP6AXXZ@Z","_get_unexpected","?terminate@@YAXXZ","?unexpected@@YAXXZ","__set_app_type","__get_app_type"};
    H module=LoadLibraryA("msvcrt.dll");if(!module)return 1;
    U words[36]={0x58433332,sizeof(H),10},used=3;
    for(U i=0;i<10;i++){SetLastError(0xface1234);H address=GetProcAddress(module,names[i]);U error=GetLastError();words[used++]=i;words[used++]=address!=0;words[used++]=error;}
    SetLastError(0xface1234);words[used++]=GetVersion();words[used++]=GetLastError();words[used++]=(U)FreeLibrary(module);
    U done=0;if(!WriteFile(GetStdHandle((U)-11),words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();return 0;
}
