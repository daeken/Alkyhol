/* Original MIT x87 ABI observer for the intrinsic reached in DAO's original
 * D3DX9 constructor. All inputs are owned numeric fixtures, never game data. */
typedef unsigned int U;typedef unsigned short W;typedef unsigned char B;typedef void *H;
struct Extended {U low,high;W exponent;};
struct Row {U index,requested;W cw_before,sw_before,cw_after,sw_after,sw_pop,reserved;B result[10],below[10];U mx_before,mx_after,error,errno_after,guard;};
struct Guarded {U before;struct Row row;U after;};
_Static_assert(sizeof(H)==4&&sizeof(struct Row)==60&&sizeof(struct Guarded)==68,"owned math record widths");
_Static_assert(__builtin_offsetof(struct Row,result)==20&&__builtin_offsetof(struct Row,below)==30&&__builtin_offsetof(struct Row,mx_before)==40,"assembly snapshot offsets");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) int * __cdecl _errno(void);
void __cdecl run_sqrt(const struct Extended *,U,struct Row *);
static void emit(const void *p,U n){U done=0;if(!WriteFile(GetStdHandle((U)-11),p,n,&done,0)||done!=n)__builtin_trap();}
U entry(void){
    static const struct Extended values[]={
        {0,0,0},{0,0,0x8000},{0,0x80000000,0x3fff},{0,0x80000000,0x4000},
        {0,0x80000000,0x4001},{0,0xc0000000,0x4000},{1,0x80000000,0x3fff},
        {0x800,0x80000000,0x3fff},{0,0x80000000,0x3bcd},{0xfffff800,0xffffffff,0x43fe},
        {0,0x80000000,0x43ff},{1,0,0},{0,0x80000000,0xbfff},
        {0,0x80000000,0x7fff},{0x1234,0xc0000000,0x7fff},{0x1234,0x80000000,0x7fff},
        {0xffffffff,0xffffffff,0x7ffe}};
    static const U precisions[]={0,0x200,0x300};
    U head[]={0x53513332,4,sizeof(struct Row),sizeof(struct Guarded),17,12};emit(head,sizeof(head));
    for(U p=0;p<3;p++)for(U rounding=0;rounding<4;rounding++)for(U i=0;i<17;i++){
        struct Guarded out;B *bytes=(B *)&out;for(U j=0;j<sizeof(out);j++)bytes[j]=0xa5;
        out.before=out.after=out.row.guard=0x715e7a11;out.row.index=i;out.row.requested=0x7f|precisions[p]|(rounding<<10);
        *_errno()=0x715e7a11;SetLastError(0xface1234);run_sqrt(&values[i],out.row.requested,&out.row);
        out.row.error=GetLastError();out.row.errno_after=(U)*_errno();emit(&out,sizeof(out));
    }
    return 0;
}
