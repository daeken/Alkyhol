/* Original MIT observer for DAO's reached non-alertable zero-timeout wait.
 * Independent guest processes measure real contention, handoff and abandonment. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) int __stdcall ReadFile(H,void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) H __stdcall CreateMutexW(H,int,const W *);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) U __stdcall WaitForSingleObjectEx(H,U,int);
__declspec(dllimport) int __stdcall ReleaseMutex(H);
#define NAME L"Local\\AlkyholOwnedMutexWait-{BF5678E1-C82F-48FA-A208-7F749BF82594}"
static U words[64],used;
static void put(U x){if(used>=64)__builtin_trap();words[used++]=x;}
static U stack(void){U x;__asm__ volatile("movl %%esp,%0":"=r"(x));return x;}
static void output(void){U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();used=0;}
static void signal(void){U done=0;char c=0;if(!ReadFile(GetStdHandle((U)-10),&c,1,&done,0)||done!=1||c!='x')__builtin_trap();}
static H create(void){SetLastError(0xface1234);H h=CreateMutexW(0,0,NAME);put(h!=0);put(GetLastError());if(!h)__builtin_trap();return h;}
static void wait(H h){SetLastError(0xface1234);put(WaitForSingleObjectEx(h,0,0));put(GetLastError());}
static void release(H h){SetLastError(0xface1234);put(ReleaseMutex(h)!=0);put(GetLastError());}
static void close(H h){SetLastError(0xface1234);put(CloseHandle(h)!=0);put(GetLastError());}
U entry(void){
    put(0x4f574d32);put(sizeof(H));U before=stack();(void)before;H a=create();
#if MODE == 0
    H b=create();release(a);wait(a);wait(b);release(a);release(b);release(b);wait(b);release(a);close(a);close(b);
    put(before==stack());put(used+1);output();
#elif MODE == 1
    wait(a);output();signal();release(a);close(a);output();
#elif MODE == 2
    wait(a);output();signal();wait(a);wait(a);release(a);release(a);close(a);output();
#elif MODE == 3
    (void)WaitForSingleObjectEx(a,0,1);__builtin_trap();
#elif MODE == 4
    (void)WaitForSingleObjectEx(a,1,0);__builtin_trap();
#endif
    return 0;
}
