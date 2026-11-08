/* Original MIT probe for the registry-open call reached by DAO startup. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall RegOpenKeyExW(H,const W *,U,U,H *);
__declspec(dllimport) U __stdcall RegCloseKey(H);
#define ROOT ((H)0x80000001)
#define PATH L"Software\\AlkyholOwnedDaoStartup\\{E97B4171-7A2B-4A82-A867-9AF56BFF70F4}"
static U words[80],used;
static __attribute__((unused)) void put(U n){if(used>=80)__builtin_trap();words[used++]=n;}
static __attribute__((unused)) U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
struct Out {U before;H value;U after;};
static struct Out out;
static void reset(void){out.before=out.after=0x715e7a11;out.value=(H)0xf00df00d;SetLastError(0xface1234);}
static __attribute__((unused)) void opened(U status){put(status);put(out.value!=0&&out.value!=(H)0xf00df00d);put(out.before==0x715e7a11&&out.after==0x715e7a11);put(GetLastError());}
U entry(void){
    reset();
#if MODE == 1
    (void)RegOpenKeyExW(ROOT,PATH,0,0x20119,&out.value);__builtin_trap();
#elif MODE == 2
    (void)RegOpenKeyExW(ROOT,PATH,0,0x20019,(H *)1);__builtin_trap();
#elif MODE == 3
    (void)RegOpenKeyExW(ROOT,L"Software\\AlkyholUnlistedScope",0,0x20019,&out.value);__builtin_trap();
#elif MODE == 4
    if(RegOpenKeyExW(ROOT,PATH,0,0x20019,&out.value))__builtin_trap();H key=out.value;(void)RegCloseKey(key);(void)RegCloseKey(key);__builtin_trap();
#else
    put(0x52473332);put(sizeof(H));U before=stack();
    U status=RegOpenKeyExW(ROOT,PATH,0,0x20019,&out.value);opened(status);if(status)__builtin_trap();H parent=out.value;
    reset();status=RegOpenKeyExW(parent,L"Present",0,1,&out.value);opened(status);if(status)__builtin_trap();H child=out.value;
    reset();status=RegOpenKeyExW(parent,L"Absent",0,0x20019,&out.value);put(status);put((U)out.value);put(out.before==0x715e7a11&&out.after==0x715e7a11);put(GetLastError());
    reset();status=RegOpenKeyExW(ROOT,PATH L"\\pReSeNt",0,0x20219,&out.value);opened(status);if(status)__builtin_trap();H again=out.value;
    SetLastError(0xface1234);put(RegCloseKey(child));put(GetLastError());put(RegCloseKey(again));put(GetLastError());put(RegCloseKey(parent));put(GetLastError());
    U after=stack();put(before==after);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
}
