/* Original MIT witness for serialized heap ownership reached by DAO startup. */
typedef unsigned int U;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) H __stdcall GetProcessHeap(void);
__declspec(dllimport) H __stdcall HeapCreate(U,U,U);
__declspec(dllimport) H __stdcall HeapAlloc(H,U,U);
__declspec(dllimport) H __stdcall HeapReAlloc(H,U,H,U);
__declspec(dllimport) U __stdcall HeapSize(H,U,H);
__declspec(dllimport) int __stdcall HeapFree(H,U,H);
__declspec(dllimport) int __stdcall HeapLock(H);
__declspec(dllimport) int __stdcall HeapUnlock(H);
static U words[32],used;
static void put(U n){if(used>=32)__builtin_trap();words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static __attribute__((unused)) void exercise(H heap){
    SetLastError(0xface1234);U before=stack();put(HeapLock(heap)!=0);put(HeapLock(heap)!=0);
    U *p=HeapAlloc(heap,8,8);put(p!=0);if(!p)__builtin_trap();p[0]=0x715e7a11;
    U *q=HeapReAlloc(heap,8,p,48);put(q!=0);if(!q)__builtin_trap();put(q[0]==0x715e7a11);
    U zero=1;for(U i=1;i<12;i++)if(q[i])zero=0;put(zero);put(HeapSize(heap,0,q));put(HeapFree(heap,0,q)!=0);
    put(HeapUnlock(heap)!=0);put(HeapUnlock(heap)!=0);U after=stack();put(before==after);put(GetLastError());
}
U entry(void){
#if MODE == 1
    (void)HeapLock((H)1);__builtin_trap();
#elif MODE == 2
    (void)HeapUnlock(GetProcessHeap());__builtin_trap();
#elif MODE == 3
    (void)HeapLock(HeapCreate(1,0,0));__builtin_trap();
#else
    put(0x484c3332);put(sizeof(H));exercise(GetProcessHeap());H heap=HeapCreate(0,0,0);put(heap!=0);if(!heap)__builtin_trap();exercise(heap);put(used+1);
    U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
}
