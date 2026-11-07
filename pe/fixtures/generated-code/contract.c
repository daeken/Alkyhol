/* Owned MIT fixture: an unchanged PE allocates, emits, executes and rewrites
 * Win64 code through real Windows memory/protection/cache/release APIs. */
typedef unsigned int U32;
typedef unsigned char U8;
typedef unsigned long long U64;
typedef U32 (__stdcall *Binary)(U32,U32);
#define DLL __declspec(dllimport)
DLL void *__stdcall VirtualAlloc(void *,U64,U32,U32);
DLL int __stdcall VirtualProtect(void *,U64,U32,U32 *);
DLL int __stdcall VirtualFree(void *,U64,U32);
DLL int __stdcall FlushInstructionCache(void *,const void *,U64);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
static U32 output[64],used;
static U32 *volatile relocated_output=output;
#define RECORD(v) do { relocated_output[used++]=(U32)(v); } while(0)
#define CHECK(c,id) do { if(!(c)) ExitProcess(id); } while(0)
static void protect(void *address,U64 size,U32 protection) {
    U32 old=0xdeadbeefu;
    int result=VirtualProtect(address,size,protection,&old);
    RECORD(result); RECORD(old); CHECK(result!=0,51);
}
U32 entry(void) {
    volatile U8 *allocation=(volatile U8 *)VirtualAlloc(0,4097,0x3000,4);
    volatile U8 *code;
    const U8 bytes[10]={0xb8,0x44,0x33,0x22,0x11,0x31,0xc8,0x01,0xd0,0xc3};
    U32 i,written=0;
    Binary call;
    RECORD(0x4a495450u); RECORD(allocation!=0); CHECK(allocation!=0,52);
    RECORD(((U64)allocation&65535u)==0);
    RECORD(allocation[64]==0&&allocation[4096]==0&&allocation[8191]==0);
    code=allocation+64;
    for(i=0;i<10;i++) code[i]=bytes[i];
    allocation[4096]=0x5a; allocation[8191]=0xa5;
    protect((void *)code,10,0x20);
    RECORD(FlushInstructionCache((void *)~(U64)0,(const void *)code,10));
    call=(Binary)(U64)code;
    RECORD(call(0x55aa0102u,0x10203040u));
    allocation[4096]=0x6b; RECORD(allocation[4096]);
    protect((void *)allocation,1,4);
    code[1]=0x98; code[2]=0xba; code[3]=0xdc; code[4]=0xfe;
    protect((void *)code,10,0x20);
    RECORD(FlushInstructionCache((void *)~(U64)0,(const void *)code,10));
    RECORD(call(0x55aa0102u,0x10203040u));
    protect((void *)(allocation+4095),2,2); /* Both pages; old is first page. */
    protect((void *)allocation,4096,4);
    protect((void *)(allocation+4096),4096,4);
    RECORD(allocation[4096]); RECORD(allocation[8191]);
    protect((void *)(allocation+4096),1,1);
    protect((void *)(allocation+4096),4096,4);
    RECORD(allocation[8191]);
    RECORD(VirtualFree((void *)allocation,0,0x8000));
    CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)&&written==used*4,53);
    return 0;
}
