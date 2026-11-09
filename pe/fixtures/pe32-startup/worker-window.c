/* Original MIT: run the unchanged owned-window witness on a real guest worker. */
#define entry window_entry
#include "window-family.c"
#undef entry
API H __stdcall CreateThread(H,U,U(__stdcall *)(void *),void *,U,U *);
API U __stdcall WaitForSingleObject(H,U);API int __stdcall GetExitCodeThread(H,U *);API int __stdcall CloseHandle(H);
static U __stdcall worker(void *parameter){(void)parameter;return window_entry();}
U entry(void){H thread=CreateThread(0,0,worker,0,0,0);if(!thread||WaitForSingleObject(thread,(U)-1)!=0)__builtin_trap();U code;if(!GetExitCodeThread(thread,&code)||code||!CloseHandle(thread))__builtin_trap();return 0;}
