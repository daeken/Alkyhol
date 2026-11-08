/* Original MIT observer for the next actual DAO CRT call. UTC observations are
 * explicitly conditional on the real native timezone, never a host-local guess. */
typedef unsigned int U;
typedef void *H;
struct FT {U low,high;};
_Static_assert(sizeof(H)==4&&sizeof(struct FT)==8,"Win32 FILETIME ABI");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetTimeZoneInformation(void *);
__declspec(dllimport) int __stdcall FileTimeToLocalFileTime(const struct FT *,struct FT *);
static void emit(const void *p,U n){U done=0;if(!WriteFile(GetStdHandle((U)-11),p,n,&done,0)||done!=n)__builtin_trap();}
U entry(void){
    struct {U info[43],guard;} tz;for(U i=0;i<43;i++)tz.info[i]=0xa5a5a5a5;tz.guard=0x715e7a11;
    U state=GetTimeZoneInformation(tz.info);U head[7]={0x544c3332,sizeof(H),state,tz.info[0],tz.info[21],tz.info[42],tz.guard};emit(head,sizeof(head));
    static const struct FT inputs[]={{0,0},{1,0},{0xd53e8000,0x019db1de},{0,0x01da0000},{0xffffffff,0x7fffffff},{0,0x80000000},{0xffffffff,0xffffffff}};
    for(U i=0;i<9;i++){
        U index=i<7?i:(i==7?2:5),alias=i>=7;
        struct {struct FT value;U guard;} source={inputs[index],0x715e7a11},target={{0xa5a5a5a5,0xa5a5a5a5},0x715e7a11};
        struct FT *out=alias?&source.value:&target.value;
        SetLastError(0xface1234);U result=(U)FileTimeToLocalFileTime(&source.value,out),error=GetLastError();
        U words[11]={i,alias,inputs[index].low,inputs[index].high,result,error,out->low,out->high,source.guard,target.guard,source.value.low==inputs[index].low&&source.value.high==inputs[index].high};emit(words,sizeof(words));
    }
    return 0;
}
