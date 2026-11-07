/* Owned MIT probe: dynamic AMD64 function-table metadata for actual generated
 * leaf code. This measures registration/lookup/removal, not exception unwinding. */
typedef unsigned int U32;
typedef unsigned char U8;
typedef unsigned long long U64;
typedef struct RuntimeFunction { U32 begin,end,unwind; } RuntimeFunction;
typedef U32 (__stdcall *Binary)(U32,U32);
_Static_assert(sizeof(RuntimeFunction)==12,"AMD64 RUNTIME_FUNCTION layout");
#define DLL __declspec(dllimport)
DLL U8 __stdcall RtlAddFunctionTable(RuntimeFunction *,U32,U64);
DLL U8 __stdcall RtlDeleteFunctionTable(RuntimeFunction *);
DLL RuntimeFunction *__stdcall RtlLookupFunctionEntry(U64,U64 *,void *);
DLL void *__stdcall VirtualAlloc(void *,U64,U32,U32);
DLL int __stdcall VirtualProtect(void *,U64,U32,U32 *);
DLL int __stdcall VirtualFree(void *,U64,U32);
DLL int __stdcall FlushInstructionCache(void *,const void *,U64);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
static RuntimeFunction table[2]={{64,74,512},{96,106,512}},other={128,138,512};
static U32 output[64],used;
static U32 *volatile relocated_output=output;
static U64 base;
#define RECORD(v) do { relocated_output[used++]=(U32)(v); } while(0)
#define CHECK(c,id) do { if(!(c)) ExitProcess(id); } while(0)
static void emit(volatile U8 *p,U32 immediate) {
    p[0]=0xb8; p[1]=(U8)immediate; p[2]=(U8)(immediate>>8);
    p[3]=(U8)(immediate>>16); p[4]=(U8)(immediate>>24);
    p[5]=0x31; p[6]=0xc8; p[7]=0x01; p[8]=0xd0; p[9]=0xc3;
}
static RuntimeFunction *lookup(U32 offset,RuntimeFunction *expected,U64 *actual_base) {
    RuntimeFunction *actual;
    *actual_base=0xaaaaaaaa55555555ull;
    actual=RtlLookupFunctionEntry(base+offset,actual_base,0);
    RECORD(actual==expected); CHECK(actual==expected,61);
    if(actual) RECORD(*actual_base==base);
    return actual;
}
static U32 call(RuntimeFunction *entry,U64 image) {
    Binary function=(Binary)(image+entry->begin);
    return function(0x55aa0102u,0x10203040u);
}
U32 entry(void) {
    volatile U8 *allocation=(volatile U8 *)VirtualAlloc(0,4096,0x3000,4);
    U32 old=0,written=0;
    U64 image;
    RuntimeFunction *found;
    RECORD(0x4654424cu); RECORD(allocation!=0); CHECK(allocation!=0,62);
    base=(U64)allocation;
    emit(allocation+64,0x11223344); emit(allocation+96,0xfedcba98); emit(allocation+128,0x10203040);
    allocation[512]=1; allocation[513]=0; allocation[514]=0; allocation[515]=0;
    /* Version-1 UNWIND_INFO, no prologue, unwind codes, frame or handlers. */
    RECORD(VirtualProtect((void *)allocation,4096,0x20,&old)); RECORD(old);
    RECORD(FlushInstructionCache((void *)~(U64)0,(const void *)allocation,516));
    lookup(64,0,&image);
    RECORD(RtlAddFunctionTable(table,2,base)); RECORD(RtlAddFunctionTable(&other,1,base));
    found=lookup(64,table,&image); RECORD(found->begin); RECORD(found->end); RECORD(found->unwind);
    RECORD(call(found,image));
    lookup(73,table,&image); lookup(74,0,&image); lookup(80,0,&image);
    found=lookup(96,table+1,&image); RECORD(call(found,image));
    found=lookup(129,&other,&image); RECORD(call(found,image));
    RECORD(RtlDeleteFunctionTable(table)); lookup(64,0,&image); lookup(96,0,&image);
    lookup(128,&other,&image); RECORD(RtlDeleteFunctionTable(table));
    RECORD(RtlDeleteFunctionTable(&other)); lookup(128,0,&image);
    RECORD(RtlAddFunctionTable(table,2,base)); lookup(96,table+1,&image);
    RECORD(RtlDeleteFunctionTable(table)); RECORD(VirtualFree((void *)allocation,0,0x8000));
    CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)&&written==used*4,63);
    return 0;
}
