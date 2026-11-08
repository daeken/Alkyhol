/* Original MIT observer for DAO's reached named, initially unowned mutex call. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) H __stdcall CreateMutexW(H,int,const W *);
__declspec(dllimport) int __stdcall CloseHandle(H);
#define NAME L"Local\\AlkyholOwnedPe32Mutex-{FFEFAC52-53A9-48CC-9660-FBD4EB3B465F}"
static U words[32],used;
static void put(U n){if(used>=32)__builtin_trap();words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
    put(0x4d583332);put(sizeof(H));U before=stack();(void)before;SetLastError(0xface1234);
#if MODE == 2
    (void)CreateMutexW(0,1,NAME);__builtin_trap();
#else
    H first=CreateMutexW(0,0,NAME);put(first!=0);put(GetLastError());if(!first)__builtin_trap();
#if MODE == 0
    SetLastError(0xface1234);H second=CreateMutexW(0,0,NAME);put(second!=0);put(GetLastError());if(!second)__builtin_trap();
    SetLastError(0xface1234);put(CloseHandle(first)!=0);put(GetLastError());SetLastError(0xface1234);put(CloseHandle(second)!=0);put(GetLastError());
    SetLastError(0xface1234);H third=CreateMutexW(0,0,NAME);put(third!=0);put(GetLastError());if(!third)__builtin_trap();SetLastError(0xface1234);put(CloseHandle(third)!=0);put(GetLastError());
#else
    SetLastError(0xface1234);put(CloseHandle(first)!=0);put(GetLastError());
#endif
    U after=stack();put(before==after);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
}
