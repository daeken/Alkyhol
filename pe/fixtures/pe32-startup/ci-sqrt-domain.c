/* Original MIT bounded intrinsic witness, including a non-double operand just
 * above a binary64 sqrt rounding threshold. The full native probe stays pinned. */
typedef unsigned int U;typedef unsigned short W;typedef unsigned char B;typedef void *H;
struct Extended {U low,high;W exponent;};
struct Row {U index,requested;W cw_before,sw_before,cw_after,sw_after,sw_pop,reserved;B result[10],below[10];U mx_before,mx_after,error,errno_after,guard;};
struct Guarded {U before;struct Row row;U after;};
_Static_assert(sizeof(H)==4&&sizeof(struct Row)==60&&sizeof(struct Guarded)==68,"owned domain record widths");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) int * __cdecl _errno(void);
void __cdecl run_sqrt(const struct Extended *,U,struct Row *);
static void emit(const void *p,U n){U done=0;if(!WriteFile(GetStdHandle((U)-11),p,n,&done,0)||done!=n)__builtin_trap();}
U entry(void){
#if BAD == 1
    static const struct Extended values[]={{0,0x80000000,0x3fff}};static const U controls[]={0x7f};
#elif BAD == 2
    static const struct Extended values[]={{0,0x80000000,0xbfff}};static const U controls[]={0x27f};
#elif BAD == 3
    static const struct Extended values[]={{0x1234,0xc0000000,0x7fff}};static const U controls[]={0x27f};
#elif BAD == 4
    static const struct Extended values[]={{0,0x80000000,0x43ff}};static const U controls[]={0x27f};
#else
    static const struct Extended values[]={
        {0,0,0},{0,0,0x8000},{0,0x80000000,0x3fff},{0,0x80000000,0x4000},
        {0,0x80000000,0x4001},{0,0xc0000000,0x4000},{1,0x80000000,0x3fff},
        {0x800,0x80000000,0x3fff},{0,0x80000000,0x3bcd},{0xfffff800,0xffffffff,0x43fe},
        {0x803,0x80000000,0x3fff},{0,0xc90fdb00,0x4000}};
    static const U controls[]={0x27f,0x37f};
#endif
    U count=sizeof(values)/sizeof(values[0]),contexts=sizeof(controls)/sizeof(controls[0]);
    U head[]={0x53513332,4,sizeof(struct Row),sizeof(struct Guarded),count,contexts};emit(head,sizeof(head));
    int *initial=_errno();if(!initial||((U)initial&3)||initial!=_errno()||*initial!=0)__builtin_trap();
    for(U c=0;c<contexts;c++)for(U i=0;i<count;i++){
        struct Guarded out;B *bytes=(B *)&out;for(U j=0;j<sizeof(out);j++)bytes[j]=0xa5;
        out.before=out.after=out.row.guard=0x715e7a11;out.row.index=i;out.row.requested=controls[c];
        *_errno()=0x715e7a11;SetLastError(0xface1234);run_sqrt(&values[i],out.row.requested,&out.row);
        out.row.error=GetLastError();out.row.errno_after=(U)*_errno();emit(&out,sizeof(out));
    }
    return 0;
}
