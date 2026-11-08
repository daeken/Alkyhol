/* Original MIT witness for the heap resizing reached by DAO's bundled CRT. */
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
static U words[96],used;
static void put(U n){if(used>=96)__builtin_trap();words[used++]=n;}
static __attribute__((unused)) U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U prefix(const unsigned char *p,U n){for(U i=0;i<n;i++)if(p[i]!=(unsigned char)(i*17+3))return 0;return 1;}
static U zeros(const unsigned char *p,U first,U end){for(U i=first;i<end;i++)if(p[i])return 0;return 1;}
static __attribute__((unused)) void exercise(H heap){
    SetLastError(0xface1234);unsigned char *p=HeapAlloc(heap,0,37);put(p!=0);if(!p)__builtin_trap();put((U)p&7);put(HeapSize(heap,0,p));put(GetLastError());
    for(U i=0;i<37;i++)p[i]=(unsigned char)(i*17+3);
    U before=stack();unsigned char *q=HeapReAlloc(heap,8,p,137);U after=stack();put(before==after);put(q!=0);if(!q)__builtin_trap();put((U)q&7);put(HeapSize(heap,0,q));put(prefix(q,37));put(zeros(q,37,137));put(GetLastError());
    p=HeapReAlloc(heap,0,q,19);put(p!=0);if(!p)__builtin_trap();put(HeapSize(heap,0,p));put(prefix(p,19));put(GetLastError());
    q=HeapReAlloc(heap,8,p,29);put(q!=0);if(!q)__builtin_trap();put(HeapSize(heap,0,q));put(prefix(q,19));put(zeros(q,19,29));put(GetLastError());
    p=HeapReAlloc(heap,16,q,7);put(p==q);if(!p)__builtin_trap();put(HeapSize(heap,0,p));put(prefix(p,7));put(GetLastError());
    put(HeapReAlloc(heap,0,p,0xffffffff)==0);put(HeapSize(heap,0,p));put(prefix(p,7));put(GetLastError());
    put(HeapFree(heap,0,p)!=0);put(GetLastError());
}
U entry(void){
    put(0x48523332);put(sizeof(H));
#if MODE == 1
    (void)HeapReAlloc(GetProcessHeap(),0,(H)1,80);
#elif MODE == 2
    H h=GetProcessHeap();H p=HeapAlloc(h,0,16);(void)HeapReAlloc(h,4,p,32);
#elif MODE == 3
    H h=GetProcessHeap(),other=HeapCreate(0,0,0);H p=HeapAlloc(h,0,16);(void)HeapReAlloc(other,0,p,32);
#else
    exercise(GetProcessHeap());H h=HeapCreate(0,0,0);put(h!=0);if(!h)__builtin_trap();exercise(h);
    put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
    __builtin_trap();
}
