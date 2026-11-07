/* Owned MIT probe: real guest GetModuleHandleW/GetProcAddress calls against the
 * unchanged frozen DLL. A single static Direct import forces native attachment. */
typedef unsigned int U32;
typedef unsigned short U16;
typedef unsigned long long U64;
typedef U32 (__stdcall *Binary)(U32,U32);
typedef U32 (__stdcall *Unary)(U32);
typedef U32 (__stdcall *Nullary)(void);
#define DLL __declspec(dllimport)
DLL U32 __stdcall Direct(U32,U32);
DLL void *__stdcall GetModuleHandleW(const U16 *);
DLL void *__stdcall GetProcAddress(void *,const char *);
DLL void __stdcall SetLastError(U32);
DLL U32 __stdcall GetLastError(void);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
static U32 output[32],used;
static U32 *volatile relocated_output=output;
#define RECORD(v) do { relocated_output[used++]=(U32)(v); } while(0)
#define CHECK(c,id) do { if(!(c)) ExitProcess(id); } while(0)
U32 entry(void) {
    U32 written=0;
    void *module=GetModuleHandleW((const U16 *)L"ALKYHOL_EXPORTS.DLL");
    Binary direct;
    Unary ordinal;
    Nullary forwarded;
    U32 *data,*by_ordinal,*bss;
    RECORD(0x44594e4cu); RECORD(module!=0); CHECK(module!=0,41);
    RECORD(GetModuleHandleW((const U16 *)L"alkyhol_exports")==module);
    RECORD(*(const U16 *)module==0x5a4d); /* Actual PE image base, not a token. */
    RECORD(Direct(7,11));
    direct=(Binary)GetProcAddress(module,"Direct"); CHECK(direct!=0,42);
    RECORD(direct(7,11));
    RECORD(GetProcAddress(module,"Alias")== (void *)direct);
    RECORD(GetProcAddress(module,(const char *)(U64)17)== (void *)direct);
    ordinal=(Unary)GetProcAddress(module,(const char *)(U64)23); CHECK(ordinal!=0,43);
    RECORD(ordinal(9));
    data=(U32 *)GetProcAddress(module,"Data"); CHECK(data!=0,44); RECORD(*data);
    by_ordinal=(U32 *)GetProcAddress(module,(const char *)(U64)21); CHECK(by_ordinal!=0,45);
    RECORD(data==by_ordinal);
    bss=(U32 *)GetProcAddress(module,"Bss"); CHECK(bss!=0,46); RECORD(*bss);
    *data=0x10203040u; RECORD(*by_ordinal); RECORD(direct(3,5));
    *data=0x31415926u;
    forwarded=(Nullary)GetProcAddress(module,"Forwarded"); CHECK(forwarded!=0,47);
    SetLastError(0x4567); RECORD(forwarded());
    SetLastError(0x1234); RECORD(GetProcAddress(module,"DefinitelyMissing")==0); RECORD(GetLastError());
    SetLastError(0x1234); RECORD(GetProcAddress(module,"direct")==0); RECORD(GetLastError());
    SetLastError(0x1234); RECORD(GetModuleHandleW((const U16 *)L"alkyhol_missing_6d89aa1.dll")==0); RECORD(GetLastError());
    CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)&&written==used*4,48);
    return 0;
}
