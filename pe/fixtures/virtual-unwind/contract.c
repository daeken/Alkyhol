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
extern U8 __ImageBase,small_body[],small_end[],small_unwind[],large_body[],large_end[],large_unwind[],leaf_body[],leaf_end[],leaf_unwind[];
extern void __stdcall small_begin(Context *),large_begin(Context *),leaf_begin(Context *);
static RuntimeFunction functions[3];
static Context context,expected;
static U32 output[64],used,seen;
static U32 *volatile relocated_output=output;
static U64 base;
#define RECORD(v) do { relocated_output[used++]=(U32)(v); } while(0)
#define CHECK(c,id) do { if(!(c)) ExitProcess(id); } while(0)
static U32 rva(const void *pointer) { return (U32)((U64)pointer-base); }
static void seed(void) {
    U32 i;
    for(i=0;i<sizeof(context);i++) ((volatile U8 *)&context)[i]=(U8)(i*13+7);
    context.flags=0x100003u; /* Integer/control fields used; not a full capture. */
}
void inspect_context(Context *actual,U32 id) {
    U32 i,equal=1;
    U64 original_rsp=actual->gpr[4],frame=0,delta=id==0?64:id==1?784:8;
    U64 *stack=(U64 *)original_rsp;
    void *handler_data=(void *)0x12345678ull,*handler;
    CHECK(id==seen&&id<3,71);
    for(i=0;i<sizeof(expected);i++) ((volatile U8 *)&expected)[i]=((volatile U8 *)actual)[i];
    if(id==0) {expected.gpr[12]=stack[5];expected.gpr[3]=stack[6];}
    if(id==1) expected.gpr[5]=stack[96];
    expected.rip=stack[delta/8-1]; expected.gpr[4]=original_rsp+delta;
    handler=RtlVirtualUnwind(0,base,actual->rip,functions+id,actual,&handler_data,&frame,0);
    RECORD(id); RECORD(handler==0); RECORD(actual->rip==expected.rip);
    RECORD(actual->gpr[4]-original_rsp); RECORD(frame==original_rsp);
    RECORD(actual->gpr[3]==expected.gpr[3]); RECORD(actual->gpr[12]==expected.gpr[12]);
    RECORD(actual->gpr[5]==expected.gpr[5]); RECORD(actual->flags);
    /* Flags are recorded exactly above; all other bytes must match the
     * independently read real stack/captured inputs, including unused state. */
    expected.flags=actual->flags;
    for(i=0;i<sizeof(expected);i++) if(((volatile U8 *)&expected)[i]!=((volatile U8 *)actual)[i]) equal=0;
    RECORD(equal);
    seen++;
}
U32 entry(void) {
    U32 written=0;
    base=(U64)&__ImageBase;
    functions[0]=(RuntimeFunction){rva(small_begin),rva(small_end),rva(small_unwind)};
    functions[1]=(RuntimeFunction){rva(large_begin),rva(large_end),rva(large_unwind)};
    functions[2]=(RuntimeFunction){rva(leaf_begin),rva(leaf_end),rva(leaf_unwind)};
    RECORD(0x56554e57u); RECORD(RtlAddFunctionTable(functions,3,base));
    seed(); small_begin(&context);
    seed(); large_begin(&context);
    seed(); leaf_begin(&context);
    RECORD(seen); RECORD(RtlDeleteFunctionTable(functions));
    CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)&&written==used*4,72);
    return 0;
}
