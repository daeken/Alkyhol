/* Original MIT observer for DAO's reached counting-semaphore constructor. */
typedef unsigned int U;typedef int L;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) H __stdcall CreateSemaphoreW(H,L,L,const W *);
__declspec(dllimport) U __stdcall WaitForSingleObjectEx(H,U,int);
__declspec(dllimport) int __stdcall CloseHandle(H);
static U words[64],used;static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static H create(L initial,L maximum){SetLastError(0xface1234);H h=CreateSemaphoreW(0,initial,maximum,0);put(h!=0);put(GetLastError());return h;}
static void wait(H h){SetLastError(0xface1234);put(WaitForSingleObjectEx(h,0,0));put(GetLastError());}
static void close(H h){SetLastError(0xface1234);put(CloseHandle(h)!=0);put(GetLastError());}
U entry(void){
    put(0x53454d32);put(sizeof(H));U before=stack();(void)before;
#if MODE == 1
    (void)CreateSemaphoreW(0,0,3,L"OwnedSemaphore");__builtin_trap();
#elif MODE == 2
    (void)CreateSemaphoreW((H)1,0,3,0);__builtin_trap();
#else
    H empty=create(0,0x7fffffff);if(!empty)__builtin_trap();
#if MODE == 3
    (void)WaitForSingleObjectEx(empty,1,0);__builtin_trap();
#elif MODE == 4
    (void)WaitForSingleObjectEx(empty,0,1);__builtin_trap();
#else
    wait(empty);H two=create(2,3);if(!two)__builtin_trap();wait(two);wait(two);wait(two);
    if(create(-1,3)||create(4,3)||create(0,0))__builtin_trap();wait(empty);close(empty);close(two);
    put(stack()==before);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
#endif
}
