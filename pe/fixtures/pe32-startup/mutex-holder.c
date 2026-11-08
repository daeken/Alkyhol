/* Original MIT fixture: real guest reference kept alive across a second process. */
#define MODE 1
#define entry unused_observer_entry
#include "mutex-create.c"
#undef entry
__declspec(dllimport) int __stdcall ReadFile(H,void *,U,U *,H);
U entry(void) {
    SetLastError(0xface1234);
    H mutex=CreateMutexW(0,0,NAME); U ready[2]={mutex!=0,GetLastError()},done=0;
    if(!mutex || !WriteFile(GetStdHandle((U)-11),ready,8,&done,0) || done!=8)__builtin_trap();
    char signal;
    if(!ReadFile(GetStdHandle((U)-10),&signal,1,&done,0) || done!=1 || signal!='x')__builtin_trap();
    if(!CloseHandle(mutex))__builtin_trap();
    return 0;
}
