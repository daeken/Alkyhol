/* Owned MIT fixture: EXE imports code, data, a NONAME ordinal and a forwarded
 * kernel function from our unchanged, independently frozen real PE DLL. */
typedef unsigned int U32;
typedef unsigned long long U64;
#define DLL __declspec(dllimport)
DLL U32 Data, Bss, DataByOrdinal;
DLL U32 __stdcall Direct(U32,U32);
DLL U32 __stdcall Alias(U32,U32);
DLL U32 __stdcall OrdinalCall(U32);
DLL U32 __stdcall Forwarded(void);
DLL void __stdcall SetLastError(U32);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
static U32 output[32],used;
static U32 *volatile relocated_output=output;
static volatile U64 data_addresses[2];
#define RECORD(v) do { relocated_output[used++]=(U32)(v); } while(0)
U32 entry(void) {
    U32 written=0;
    RECORD(0x4c455845u);
    RECORD(Data); RECORD(Bss);
    RECORD(Direct(7,11)); RECORD(Alias(7,11)); RECORD(OrdinalCall(9));
    RECORD(DataByOrdinal);
    /* Distinct extern C object names may be folded unequal by the compiler.
     * Observe the actual integer addresses through volatile storage instead. */
    data_addresses[0]=(U64)&DataByOrdinal; data_addresses[1]=(U64)&Data;
    RECORD(data_addresses[0]==data_addresses[1]);
    SetLastError(0x3456); RECORD(Forwarded());
    Bss^=0x55aa55aau; RECORD(Direct(7,11));
    DataByOrdinal=0x10203040u; RECORD(Data); RECORD(Direct(3,5));
    Data=0x31415926u; Bss=0x13579bdfu;
    RECORD(Direct(7,11)); RECORD(Bss);
    if(!WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)||written!=used*4)
        ExitProcess(33);
    return 0;
}
