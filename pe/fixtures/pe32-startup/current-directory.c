/* Original MIT microtest for the reached DAO SetCurrentDirectoryW call.
 * Actual directory changes must redirect real relative reads; failures preserve
 * the preceding directory. No game files, paths or application names enter here. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
_Static_assert(sizeof(H)==4&&sizeof(W)==2&&sizeof(U)==4,"Win32 widths");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) int __stdcall ReadFile(H,void *,U,U *,H);
__declspec(dllimport) H __stdcall CreateFileW(const W *,U,U,H,U,U,H);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W *,U);
__declspec(dllimport) U __stdcall GetCurrentDirectoryW(U,W *);
__declspec(dllimport) int __stdcall SetCurrentDirectoryW(const W *);
static U words[128],used;
static W base[512],a[512],b[512],missing[512],file[512],trailing[512];
static void put(U n){if(used>=128)__builtin_trap();words[used++]=n;}
static U length(const W *s){U n=0;while(s[n]){if(n>=510)__builtin_trap();n++;}return n;}
static void join(W *out,const W *parent,const W *child){U at=0;for(;parent[at];at++){if(at>=510)__builtin_trap();out[at]=parent[at];}out[at++]='\\';for(U i=0;child[i];i++){if(at>=511)__builtin_trap();out[at++]=child[i];}out[at]=0;}
static int equal(const W *a,const W *b){for(U i=0;i<512;i++){if(a[i]!=b[i])return 0;if(!a[i])return 1;}__builtin_trap();}
static void change(const W *path){SetLastError(0xface1234);put(SetCurrentDirectoryW(path)!=0);put(GetLastError());}
static void observe(const W *expected){
    struct {U before;W text[512];U after;} output;
    output.before=output.after=0x715e7a11;for(U i=0;i<512;i++)output.text[i]=0x7171;
    SetLastError(0xface1234);put(GetCurrentDirectoryW(0,0));put(GetLastError());
    SetLastError(0xface1234);put(GetCurrentDirectoryW(1,output.text));put(output.text[0]);put(GetLastError());
    SetLastError(0xface1234);U got=GetCurrentDirectoryW(512,output.text);put(got);put(GetLastError());
    if(got>=512)__builtin_trap();put(equal(expected,output.text));put(output.text[got]);put(output.before);put(output.after);
    SetLastError(0xface1234);H h=CreateFileW(L"sentinel.bin",0x80000000,7,0,3,128,0);put(h!=(H)-1);put(GetLastError());if(h==(H)-1)__builtin_trap();
    U done=0;unsigned char byte=0;put(ReadFile(h,&byte,1,&done,0)!=0);put(done);put(byte);
    SetLastError(0xface1234);put(CloseHandle(h)!=0);put(GetLastError());
}
U entry(void){
    put(0x43445732);put(sizeof(H));
    U n=GetModuleFileNameW(0,base,512);if(!n||n>=512)__builtin_trap();
    while(n&&base[n]!='\\')n--;if(!n)__builtin_trap();base[n]=0;
    join(a,base,L"cwd-owned-λ🌸");join(b,a,L"inner");join(missing,b,L"missing");join(file,b,L"sentinel.bin");join(trailing,a,L"");
#if MODE == 1
    change(L"relative-directory");__builtin_trap();
#elif MODE == 2
    change(L"C:\\unknown-profile");__builtin_trap();
#else
    change(trailing);observe(a);change(b);observe(b);change(missing);change(file);observe(b);
    put(length(a));put(length(b));put(used+1);U done=0;
    if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
}
