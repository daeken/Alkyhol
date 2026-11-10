/* Owned MIT: original worker window plus concurrent CPU and host-event polling. */
#define entry window_entry
#include "../../pe/fixtures/pe32-startup/window-family.c"
#undef entry
API H __stdcall CreateThread(H,U,U(__stdcall *)(void *),void *,U,U *);
API U __stdcall WaitForSingleObject(H,U);
API int __stdcall GetExitCodeThread(H,U *);
API int __stdcall CloseHandle(H);
API int __stdcall PeekMessageW(void *,H,U,U,U);
static U finished,counter,progress;
static U __stdcall compute(void *p){(void)p;while(!__atomic_load_n(&finished,__ATOMIC_ACQUIRE)){
 __atomic_fetch_add(&counter,1,__ATOMIC_RELAXED);(void)GetLastError();
}return 0;}
static U __stdcall gui(void *p){(void)p;U result=window_entry();U msg[7];
 U before=__atomic_load_n(&counter,__ATOMIC_ACQUIRE);
 for(U i=0;i<256;++i)(void)PeekMessageW(msg,0,0,0,1);
 progress=__atomic_load_n(&counter,__ATOMIC_ACQUIRE)>before;
 __atomic_store_n(&finished,1,__ATOMIC_RELEASE);return result;}
U entry(void){U a=99,b=99;H worker=CreateThread(0,0,compute,0,0,0),window=CreateThread(0,0,gui,0,0,0);
 if(!worker||!window||WaitForSingleObject(window,(U)-1)||WaitForSingleObject(worker,(U)-1))__builtin_trap();
 if(!GetExitCodeThread(worker,&a)||!GetExitCodeThread(window,&b)||!CloseHandle(window)||!CloseHandle(worker))__builtin_trap();
 U out[]={0x5045454b,a==0,b==0,progress},done=0;
 if(!WriteFile(GetStdHandle((U)-11),out,sizeof(out),&done,0)||done!=sizeof(out))__builtin_trap();return 0;}
