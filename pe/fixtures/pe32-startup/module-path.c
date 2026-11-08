/* Original MIT targeted test for the original DAO CRT's module/path calls.
 * Paths differ per machine; compare contract, suffix, UTF-16, guards and errors,
 * never pretend the full Windows/host path strings should be identical. */
typedef unsigned int U32;
typedef unsigned short W;
typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U32,U32 *,H);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall GetModuleHandleW(const W *);
__declspec(dllimport) U32 __stdcall GetModuleFileNameW(H,W *,U32);
__declspec(dllimport) U32 __stdcall GetLongPathNameW(const W *,W *,U32);
__declspec(dllimport) H __stdcall SourceModule(void);
static void emit(const U32 *data,U32 count) {
    U32 written=0;if(!WriteFile(GetStdHandle((U32)-11),data,count*4,&written,0)||written!=count*4)__builtin_trap();
}
#if MODE==0 || MODE==4
static W full[2048],other[2048];
static W paths[3][2048];static U32 path_lengths[3],path_index;
static U32 words[128],used;
static void put(U32 value) {if(used>=128)__builtin_trap();words[used++]=value;}
static void fill(W *p) {for(U32 i=0;i<2048;i++)p[i]=0xa55a;}
static U32 equal(const W *a,const W *b,U32 n) {for(U32 i=0;i<n;i++)if(a[i]!=b[i])return 0;return 1;}
static U32 suffix(const W *p,U32 n,const char *s) {
    U32 k=0;while(s[k])k++;if(n<k)return 0;for(U32 i=0;i<k;i++)if(p[n-k+i]!=(W)s[i])return 0;return 1;
}
static void test(H module,const char *tail) {
    fill(full);SetLastError(0xface1234);
    U32 n=GetModuleFileNameW(module,full,2048),error=GetLastError();
    if(!n||n>=2047)__builtin_trap();
    if(path_index>=3)__builtin_trap();path_lengths[path_index]=n;
    volatile W *copy=paths[path_index++];for(U32 i=0;i<=n;i++)copy[i]=full[i];
    put(full[n]==0&&full[n+1]==0xa55a);put(error);put(suffix(full,n,tail));
    U32 unicode=0;for(U32 i=0;i+2<n;i++)if(full[i]==0x3bb&&full[i+1]==0xd83c&&full[i+2]==0xdf38)unicode=1;
    put(unicode);put(full[1]==':'&&full[2]=='\\');
    fill(other);SetLastError(0xface1234);put(GetModuleFileNameW(module,other,n+1)==n);put(GetLastError());put(equal(full,other,n+1)&&other[n+1]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetModuleFileNameW(module,other,n)==n);put(GetLastError());put(equal(full,other,n-1)&&other[n-1]==0);put(other[n]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetModuleFileNameW(module,other,5));put(GetLastError());put(equal(full,other,4)&&other[4]==0);put(other[5]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetModuleFileNameW(module,other,1));put(GetLastError());put(other[0]==0&&other[1]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetModuleFileNameW(module,other,0));put(GetLastError());put(other[0]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetLongPathNameW(full,other,2048)==n);put(GetLastError());put(equal(full,other,n+1)&&other[n+1]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetLongPathNameW(full,other,0)==n+1);put(GetLastError());put(other[0]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetLongPathNameW(full,other,1)==n+1);put(GetLastError());put(other[0]==0xa55a&&other[1]==0xa55a);
    SetLastError(0xface1234);put(GetLongPathNameW(full,full,2048)==n);put(GetLastError());put(suffix(full,n,tail)&&full[n]==0&&full[n+1]==0xa55a);
}
#endif
U32 entry(void) {
    H dll=SourceModule();if(!dll)__builtin_trap();
    U32 boot=0x4d503332;emit(&boot,1);
#if MODE==0 || MODE==4
    const W dll_name[]={'m','o','d','u','l','e','-','p','a','t','h','.','d','l','l',0};
    put(sizeof(void *));put(GetModuleHandleW(dll_name)==dll);
    const char *tail=MODE==4?"module-path4.exe":"module-path0.exe";
    test(0,tail);test(GetModuleHandleW(0),tail);test(dll,"module-path.dll");emit(words,used);
    for(U32 i=0;i<3;i++){emit(&path_lengths[i],1);U32 written=0;
        if(!WriteFile(GetStdHandle((U32)-11),paths[i],(path_lengths[i]+1)*2,&written,0)||written!=(path_lengths[i]+1)*2)__builtin_trap();}
#elif MODE==1
    static const W readonly[8]={0};(void)GetModuleFileNameW(dll,(W *)readonly,8);emit(&boot,1);
#elif MODE==2
    W out[8];(void)GetModuleFileNameW((H)0xdead0000,out,8);emit(&boot,1);
#else
    static const W unknown[]={'Z',':','\\','u','n','r','e','g','i','s','t','e','r','e','d','.','d','l','l',0};
    W out[8];(void)GetLongPathNameW(unknown,out,8);emit(&boot,1);
#endif
    return 0;
}
