/* Original MIT host-restriction case: fail after a real dependency initialized. */
typedef unsigned int U32;
__declspec(dllimport) U32 Data;
__declspec(dllimport) U32 __cdecl RunCdecl(U32,U32,U32);
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
int __stdcall DllMain(void *module,U32 reason,void *reserved) {
    U32 result=RunCdecl(1,2,reason);
    U32 words[6]={0x4641494c,reason,reserved!=0,Data,result,module!=0};
    U32 written=0;
    if(!WriteFile(GetStdHandle((U32)-11),words,sizeof(words),&written,0)||written!=sizeof(words)) __builtin_trap();
    return 0;
}
__declspec(dllexport) U32 __cdecl Run(U32 a,U32 b,U32 c) { return a^b^c; }
