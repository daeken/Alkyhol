/* Separate owned real-storage witness; the original SYSTEM_INFO producer stays pinned. */
#define entry original_observer_entry
#include "system-info.c"
#undef entry
__declspec(dllimport) H __stdcall GetModuleHandleA(const char *);
__declspec(dllimport) W *__stdcall GetCommandLineW(void);
__declspec(dllimport) H __stdcall HeapCreate(U,U,U);
__declspec(dllimport) H __stdcall HeapAlloc(H,U,U);
__declspec(dllimport) int __stdcall HeapFree(H,U,H);
U entry(void){
    guarded.before=guarded.after=0x715e7a11;SetLastError(0xface1234);U before=stack();GetSystemInfo(&guarded.value);U error=GetLastError();
    H module=GetModuleHandleA(0);W *command=GetCommandLineW();H heap=HeapCreate(0,0,0);volatile unsigned char *p=HeapAlloc(heap,0,65540);if(!p)__builtin_trap();p[0]=0x71;p[65539]=0x19;
    U first=p[0],last=p[65539],released=HeapFree(heap,0,(H)p)!=0;U after=stack();
    U out[18]={0x424d3332,4,(U)guarded.value.minimum,(U)guarded.value.maximum,(U)module,(U)&before,(U)&guarded.value,(U)p,65540,(U)command,first,last,released,before==after,error,guarded.before,guarded.after,18};
    U done=0;if(!WriteFile(GetStdHandle((U)-11),out,sizeof(out),&done,0)||done!=sizeof(out))__builtin_trap();return 0;
}
