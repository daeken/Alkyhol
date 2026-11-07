/* Owned MIT observer. Curated partial-prologue CONTEXT records use actual
 * saved words of a live full frame and linker-generated unwind metadata.
 * Partial states are constructed, not CPU snapshots of interrupted execution.
 * The actual full frame returns normally; no exception or resumption occurs. */
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
_Static_assert(__builtin_offsetof(Context,rest)+256==512,"Xmm6 offset");
#define DLL __declspec(dllimport)
DLL RuntimeFunction *__stdcall RtlLookupFunctionEntry(U64,U64 *,void *);
DLL void *__stdcall RtlVirtualUnwind(U32,U64,U64,RuntimeFunction *,Context *,void **,U64 *,void *);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
extern U8 __ImageBase,target_begin[],after_rbx[],after_r12[],after_alloc[],after_save[],target_body[],target_end[];
extern void __stdcall target_begin_call(Context *) __asm__("target_begin");
static Context context,actual,expected;
static U32 output[128],used;
static U32 *volatile relocated_output=output;
static U64 base;
#define RECORD(v) do {relocated_output[used++]=(U32)(v);} while(0)
#define CHECK(c,id) do {if(!(c))ExitProcess(id);} while(0)
static RuntimeFunction *lookup(U64 pc) {
 U64 returned=0;RuntimeFunction *entry=RtlLookupFunctionEntry(pc,&returned,0);
 CHECK(entry!=0&&returned==base,71);return entry;
}
void inspect_context(Context *snapshot) {
 U8 *points[6]={target_begin,after_rbx,after_r12,after_alloc,after_save,target_body};
 U32 offsets[6]={184,176,168,0,0,0},i,j,equal;
 U64 rsp=snapshot->gpr[4],*stack=(U64 *)rsp;
 RECORD(0x50524f4cu);RECORD(6);
 for(i=0;i<6;i++) {
  U64 frame=0,start=rsp+offsets[i];RuntimeFunction *row=lookup((U64)points[i]);void *data=0,*handler;
  for(j=0;j<sizeof(actual);j++)((volatile U8 *)&actual)[j]=((volatile U8 *)snapshot)[j];
  actual.gpr[4]=start;actual.rip=(U64)points[i];
  /* Unexecuted saves have not lost the caller's register value. Executed
   * saves intentionally use clobbered register seeds to test restoration. */
  if(i<1)actual.gpr[3]=stack[22];
  if(i<2)actual.gpr[12]=stack[21];
  if(i<4)actual.gpr[13]=stack[5];
  if(i<5)for(j=0;j<16;j++)actual.rest[256+j]=((U8 *)stack)[64+j];
  for(j=0;j<sizeof(expected);j++)((volatile U8 *)&expected)[j]=((volatile U8 *)&actual)[j];
  expected.gpr[3]=stack[22];expected.gpr[12]=stack[21];expected.gpr[13]=stack[5];
  for(j=0;j<16;j++)expected.rest[256+j]=((U8 *)stack)[64+j];
  expected.gpr[4]=rsp+192;expected.rip=stack[23];
  handler=RtlVirtualUnwind(0,base,actual.rip,row,&actual,&data,&frame,0);
  RECORD((U64)points[i]-(U64)target_begin);RECORD(handler==0);RECORD(actual.rip==expected.rip);
  RECORD(actual.gpr[4]-start);RECORD(frame==start);
  RECORD(actual.gpr[3]==expected.gpr[3]);RECORD(actual.gpr[12]==expected.gpr[12]);
  RECORD(actual.gpr[13]==expected.gpr[13]);
  equal=1;for(j=0;j<16;j++)if(actual.rest[256+j]!=expected.rest[256+j])equal=0;
  RECORD(equal);RECORD(actual.flags);expected.flags=actual.flags;
  equal=1;for(j=0;j<sizeof(expected);j++)if(((volatile U8 *)&expected)[j]!=((volatile U8 *)&actual)[j])equal=0;
  RECORD(equal);
 }
}
U32 entry(void) {
 U32 i,written=0;base=(U64)&__ImageBase;
 for(i=0;i<sizeof(context);i++)((volatile U8 *)&context)[i]=(U8)(i*13+7);
 context.flags=0x100003u;target_begin_call(&context);
 CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)&&written==used*4,72);
 return 0;
}
