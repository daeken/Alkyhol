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
DLL RuntimeFunction *__stdcall RtlLookupFunctionEntry(U64,U64 *,void *);
DLL U8 __stdcall RtlDeleteFunctionTable(RuntimeFunction *);
DLL void *__stdcall RtlVirtualUnwind(U32,U64,U64,RuntimeFunction *,Context *,void **,U64 *,void *);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
extern U8 __ImageBase,target_begin[],target_body[],target_end[];
extern void __stdcall target_begin_call(Context *) __asm__("target_begin");
static Context context,expected;
static U32 output[64],used;
static U32 *volatile relocated_output=output;
static U64 base;
#define RECORD(v) do {relocated_output[used++]=(U32)(v);} while(0)
#define CHECK(c,id) do {if(!(c))ExitProcess(id);} while(0)
static RuntimeFunction *lookup(U64 pc) {
 U64 returned=0;RuntimeFunction *entry=RtlLookupFunctionEntry(pc,&returned,0);
 CHECK(entry!=0&&returned==base,71);return entry;
}
void inspect_context(Context *actual) {
 U32 i,equal=1;U64 rsp=actual->gpr[4],frame=0;U64 *stack=(U64 *)rsp;
 RuntimeFunction *entry=lookup(actual->rip);void *handler_data=0,*handler;
 for(i=0;i<sizeof(expected);i++)((volatile U8 *)&expected)[i]=((volatile U8 *)actual)[i];
 expected.gpr[12]=stack[5];expected.gpr[3]=stack[6];expected.rip=stack[7];expected.gpr[4]=rsp+64;
 handler=RtlVirtualUnwind(0,base,actual->rip,entry,actual,&handler_data,&frame,0);
 RECORD(handler==0);RECORD(actual->rip==expected.rip);RECORD(actual->gpr[4]-rsp);RECORD(frame==rsp);
 RECORD(actual->gpr[3]==expected.gpr[3]);RECORD(actual->gpr[12]==expected.gpr[12]);RECORD(actual->flags);
 expected.flags=actual->flags;
 for(i=0;i<sizeof(expected);i++)if(((volatile U8 *)&expected)[i]!=((volatile U8 *)actual)[i])equal=0;
 RECORD(equal);
}
U32 entry(void) {
 U32 i,written=0,pe,dir_rva,dir_size;RuntimeFunction *row;U64 ignored=0;
 base=(U64)&__ImageBase;
 pe=*(U32 *)(base+0x3c);dir_rva=*(U32 *)(base+pe+24+112+3*8);dir_size=*(U32 *)(base+pe+24+116+3*8);
 row=lookup((U64)target_body);
 RECORD(0x53544154u);RECORD(dir_size/12);RECORD((U64)row==base+dir_rva);
 RECORD(row->begin==(U64)target_begin-base);RECORD(row->end==(U64)target_end-base);
 RECORD(row->unwind!=0&&row->unwind%4==0);
 RECORD(lookup((U64)target_begin)==row);RECORD(lookup((U64)target_end-1)==row);
 RECORD(RtlLookupFunctionEntry((U64)target_end,&ignored,0)==0);
 RECORD(RtlDeleteFunctionTable(row));RECORD(lookup((U64)target_body)==row);
 for(i=0;i<sizeof(context);i++)((volatile U8 *)&context)[i]=(U8)(i*13+7);
 context.flags=0x100003u;target_begin_call(&context);
 RECORD(lookup((U64)target_body)==row);
 CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)&&written==used*4,72);
 return 0;
}
