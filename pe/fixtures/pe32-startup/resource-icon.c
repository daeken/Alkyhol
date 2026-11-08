/* Original MIT LoadIcon observer for DAO's reached resource-backed icon call. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) H __stdcall GetModuleHandleW(const W *);
__declspec(dllimport) H __stdcall LoadIconW(H,const W *);
static U words[16],used;
static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
    H module=GetModuleHandleW(0);(void)module;U before=stack();(void)before;
#if MODE == 1
    (void)LoadIconW(0,(const W *)32512);__builtin_trap();
#elif MODE == 2
    (void)LoadIconW(module,L"OwnedName");__builtin_trap();
#else
    put(0x49434f32);put(sizeof(H));SetLastError(0xface1234);
    H first=LoadIconW(module,(const W *)41);put(first!=0);put(GetLastError());if(!first)__builtin_trap();
    SetLastError(0xface1234);put(LoadIconW(module,(const W *)41)==first);put(GetLastError());
    SetLastError(0xface1234);put(LoadIconW(module,(const W *)499)==0);put(GetLastError());
    put(stack()==before);put(used+1);U done=0;
    if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
}
