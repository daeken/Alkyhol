/* Owned MIT probe. Manually unwind captured fields of actual active AMD64
 * assembly frames. The actual stack still returns normally; no throw/catch. */
typedef unsigned int U32;
typedef unsigned short U16;
typedef unsigned char U8;
typedef unsigned long long U64;
typedef struct RuntimeFunction { U32 begin,end,unwind; } RuntimeFunction;
typedef struct __declspec(align(16)) Context {
    U64 homes[6]; U32 flags,mxcsr; U16 segments[6]; U32 eflags;
    U64 debug[6],gpr[16],rip; U8 rest[976];
} Context;
_Static_assert(sizeof(Context)==1232,"AMD64 CONTEXT size");
_Static_assert(__alignof(Context)==16,"AMD64 CONTEXT alignment");
_Static_assert(__builtin_offsetof(Context,gpr)+4*8==152,"Rsp offset");
_Static_assert(__builtin_offsetof(Context,rip)==248,"Rip offset");
#define DLL __declspec(dllimport)
DLL U8 __stdcall RtlAddFunctionTable(RuntimeFunction *,U32,U64);
DLL U8 __stdcall RtlDeleteFunctionTable(RuntimeFunction *);
DLL void *__stdcall RtlVirtualUnwind(U32,U64,U64,RuntimeFunction *,Context *,void **,U64 *,void *);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
extern U8 __ImageBase,gpr_body[],gpr_end[],gpr_unwind[],xmm_body[],xmm_end[],xmm_unwind[],far_body[],far_end[],far_unwind[];
extern void __stdcall gpr_begin(Context *),xmm_begin(Context *),far_begin(Context *);
static RuntimeFunction functions[3];
static Context context,expected;
static U32 output[64],used,seen;
static U32 *volatile relocated_output=output;
static U64 base;
#define RECORD(v) do { relocated_output[used++]=(U32)(v); } while(0)
#define CHECK(c,id) do { if(!(c)) ExitProcess(id); } while(0)
static void copy_bytes(void *target,const void *source,U32 count) {
    U32 i; for(i=0;i<count;i++) ((volatile U8 *)target)[i]=((const volatile U8 *)source)[i];
}
static U32 equal_bytes(const void *left,const void *right,U32 count) {
    U32 i; for(i=0;i<count;i++) if(((const volatile U8 *)left)[i]!=((const volatile U8 *)right)[i]) return 0;
    return 1;
}
static U32 rva(const void *pointer) { return (U32)((U64)pointer-base); }
static void seed(void) {
    U32 i;
    for(i=0;i<sizeof(context);i++) ((volatile U8 *)&context)[i]=(U8)(i*13+7);
    context.flags=0x10000bu; /* Integer/control/XMM inputs; not a full capture. */
}
void inspect_context(Context *actual,U32 id) {
    U32 equal;
    U64 original_rsp=actual->gpr[4],frame=0,delta=id==2?1048704:128;
    U8 *stack=(U8 *)original_rsp;
    U32 gpr_offset=id==2?524288:64,xmm_offset=id==2?1048576:64;
    void *handler_data=(void *)0x12345678ull,*handler;
    CHECK(id==seen&&id<3,71);
    copy_bytes(&expected,actual,sizeof(expected));
    if(id!=1) {
        copy_bytes(&expected.gpr[3],stack+gpr_offset,8);
        copy_bytes(&expected.gpr[12],stack+gpr_offset+16,8);
        CHECK(actual->gpr[3]!=expected.gpr[3]&&actual->gpr[12]!=expected.gpr[12],73);
    }
    if(id!=0) {
        copy_bytes((U8 *)&expected+512,stack+xmm_offset,16);
        copy_bytes((U8 *)&expected+656,stack+xmm_offset+32,16);
        CHECK(!equal_bytes((U8 *)actual+512,(U8 *)&expected+512,16),74);
        CHECK(!equal_bytes((U8 *)actual+656,(U8 *)&expected+656,16),75);
    }
    copy_bytes(&expected.rip,stack+delta-8,8); expected.gpr[4]=original_rsp+delta;
    handler=RtlVirtualUnwind(0,base,actual->rip,functions+id,actual,&handler_data,&frame,0);
    RECORD(id); RECORD(handler==0); RECORD(actual->rip==expected.rip);
    RECORD(actual->gpr[4]-original_rsp); RECORD(frame==original_rsp);
    RECORD(actual->gpr[3]==expected.gpr[3]); RECORD(actual->gpr[12]==expected.gpr[12]);
    RECORD(equal_bytes((U8 *)actual+512,(U8 *)&expected+512,16));
    RECORD(equal_bytes((U8 *)actual+656,(U8 *)&expected+656,16)); RECORD(actual->flags);
    expected.flags=actual->flags;
    equal=equal_bytes(&expected,actual,sizeof(expected)); RECORD(equal);
    seen++;
}
U32 entry(void) {
    U32 written=0;
    base=(U64)&__ImageBase;
    functions[0]=(RuntimeFunction){rva(gpr_begin),rva(gpr_end),rva(gpr_unwind)};
    functions[1]=(RuntimeFunction){rva(xmm_begin),rva(xmm_end),rva(xmm_unwind)};
    functions[2]=(RuntimeFunction){rva(far_begin),rva(far_end),rva(far_unwind)};
    RECORD(0x45564153u); RECORD(RtlAddFunctionTable(functions,3,base));
    seed(); gpr_begin(&context);
    seed(); xmm_begin(&context);
    seed(); far_begin(&context);
    RECORD(seen); RECORD(RtlDeleteFunctionTable(functions));
    CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)&&written==used*4,72);
    return 0;
}
