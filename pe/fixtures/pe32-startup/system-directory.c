/* Original MIT targeted observer for the actual DAO CRT system-directory call.
 * The real per-host directory is checked separately from copy/size semantics. */
typedef unsigned int U32;
typedef unsigned short W;
typedef void *H;
_Static_assert(sizeof(H)==4&&sizeof(W)==2,"Win32 widths");
__declspec(dllimport) H __stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U32,U32 *,H);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) U32 __stdcall GetSystemDirectoryW(W *,U32);
static void emit(const void *data,U32 bytes) {
    U32 written=0;if(!WriteFile(GetStdHandle((U32)-11),data,bytes,&written,0)||written!=bytes)__builtin_trap();
}
#if MODE==0
static W full[1024],other[1024];
static U32 words[32],used;
static void put(U32 value) {if(used>=32)__builtin_trap();words[used++]=value;}
static void fill(W *p) {for(U32 i=0;i<1024;i++)p[i]=0xa55a;}
static U32 equal(const W *a,const W *b,U32 n) {for(U32 i=0;i<n;i++)if(a[i]!=b[i])return 0;return 1;}
#endif
U32 entry(void) {
    U32 boot=0x53443332;emit(&boot,4);
#if MODE==0
    put(sizeof(H));fill(full);SetLastError(0xface1234);
    U32 n=GetSystemDirectoryW(full,1024),error=GetLastError();if(!n||n>=1023)__builtin_trap();
    put(full[n]==0&&full[n+1]==0xa55a);put(error);put(full[1]==':'&&full[2]=='\\');put(full[n-1]!='\\');
    fill(other);SetLastError(0xface1234);put(GetSystemDirectoryW(other,n+1)==n);put(GetLastError());put(equal(full,other,n+1)&&other[n+1]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetSystemDirectoryW(other,n)==n+1);put(GetLastError());put(other[0]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetSystemDirectoryW(other,1)==n+1);put(GetLastError());put(other[0]==0xa55a);
    fill(other);SetLastError(0xface1234);put(GetSystemDirectoryW(other,0)==n+1);put(GetLastError());put(other[0]==0xa55a);
    emit(words,used*4);emit(&n,4);emit(full,(n+1)*2);
#elif MODE==2
    static const W readonly[1024]={0};(void)GetSystemDirectoryW((W *)readonly,1024);emit(&boot,4);
#else
    W out[8];(void)GetSystemDirectoryW(out,8);emit(&boot,4);
#endif
    return 0;
}
