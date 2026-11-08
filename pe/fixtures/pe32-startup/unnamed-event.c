/* Original MIT observer for DAO's reached CreateEventW constructor.
 * Real manual/automatic states, zero-timeout consumption, reset and close. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) H __stdcall CreateEventW(H,int,int,const W *);
__declspec(dllimport) int __stdcall SetEvent(H);
__declspec(dllimport) int __stdcall ResetEvent(H);
__declspec(dllimport) U __stdcall WaitForSingleObjectEx(H,U,int);
__declspec(dllimport) int __stdcall CloseHandle(H);
static U words[64],used;
static void put(U n){if(used>=64)__builtin_trap();words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static void wait(H h){SetLastError(0xface1234);put(WaitForSingleObjectEx(h,0,0));put(GetLastError());}
static void change(H h,int set){SetLastError(0xface1234);put((set?SetEvent(h):ResetEvent(h))!=0);put(GetLastError());}
static H create(int manual,int initial){SetLastError(0xface1234);H h=CreateEventW(0,manual,initial,0);put(h!=0);put(GetLastError());if(!h)__builtin_trap();return h;}
static void close(H h){SetLastError(0xface1234);put(CloseHandle(h)!=0);put(GetLastError());}
U entry(void){
    put(0x45563332);put(sizeof(H));U before=stack();(void)before;
#if MODE == 1
    (void)CreateEventW(0,1,0,L"OwnedEvent");__builtin_trap();
#elif MODE == 2
    (void)CreateEventW((H)1,1,0,0);__builtin_trap();
#else
    H manual=create(1,0);
#if MODE == 3
    (void)WaitForSingleObjectEx(manual,1,0);__builtin_trap();
#elif MODE == 4
    (void)WaitForSingleObjectEx(manual,0,1);__builtin_trap();
#elif MODE == 5
    (void)CloseHandle(manual);(void)SetEvent(manual);__builtin_trap();
#else
    H automatic=create(0,1);wait(manual);wait(automatic);wait(automatic);change(manual,1);wait(manual);wait(manual);
    change(manual,0);change(manual,0);wait(manual);change(automatic,1);change(automatic,1);wait(automatic);wait(automatic);
    change(automatic,0);wait(automatic);close(manual);close(automatic);
    put(before==stack());put(used+1);U done=0;
    if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
#endif
}
