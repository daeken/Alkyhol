/* Original MIT observer for the FileTimeToSystemTime call actually reached by
 * DAO's CRT. Every source/output guard is retained, including invalid ranges. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
struct FT {U low,high;};struct ST {W fields[8];};
_Static_assert(sizeof(H)==4&&sizeof(struct FT)==8&&sizeof(struct ST)==16,"Win32 time widths");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) int __stdcall FileTimeToSystemTime(const struct FT *,struct ST *);
static void emit(const void *p,U n){U done=0;if(!WriteFile(GetStdHandle((U)-11),p,n,&done,0)||done!=n)__builtin_trap();}
U entry(void){
    static const struct FT values[]={{0,0},{1,0},{0xd53e8000,0x019db1de},{0xc43f8000,0x014f6598},{0xebcc8000,0x01bf8247},{0x675298f0,0x01da6b6b},{0xffffffff,0x7fffffff},{0,0x80000000},{0xffffffff,0xffffffff}};
    U head[3]={0x53543332,sizeof(H),sizeof(struct ST)};emit(head,sizeof(head));
    for(U i=0;i<9;i++){
        struct {struct FT value;U guard;} in={values[i],0x715e7a11};
        struct {struct ST value;U guard;} out;for(U j=0;j<8;j++)out.value.fields[j]=0xa5a5;out.guard=0x715e7a11;
        SetLastError(0xface1234);U result=(U)FileTimeToSystemTime(&in.value,&out.value),error=GetLastError();
        U words[8]={i,values[i].low,values[i].high,result,error,in.guard,out.guard,in.value.low==values[i].low&&in.value.high==values[i].high};emit(words,sizeof(words));emit(&out.value,sizeof(out.value));
    }
    return 0;
}
