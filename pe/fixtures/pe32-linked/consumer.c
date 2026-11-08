/* Original MIT owned Win32 DLL: real named/ordinal code and shared-data imports. */
typedef unsigned int U32;
__declspec(dllimport) U32 __stdcall RunStdcall(U32,U32,U32,U32,U32);
__declspec(dllimport) U32 __cdecl RunCdecl(U32,U32,U32);
__declspec(dllimport) U32 Data;
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
static void emit(U32 *words,U32 count) {
    U32 written=0;
    if(!WriteFile(GetStdHandle((U32)-11),words,count*4,&written,0)||written!=count*4)
        __builtin_trap();
}
int __stdcall DllMain(void *module,U32 reason,void *reserved) {
    (void)module;
    U32 before=Data;
    Data^=0x01020304;
    U32 result=reason==1?RunStdcall(1,2,3,4,5):RunStdcall(5,4,3,2,1);
    U32 words[8]={0x50323349,reason,reserved==0,before,result,Data,sizeof(void *),0xa1b2c3d4};
    emit(words,8);
    return 1;
}
__declspec(dllexport) U32 __cdecl Run(U32 a,U32 b,U32 c) {
    Data=0x2468ace0;
    U32 first=RunCdecl(a,b,c);
    U32 second=RunStdcall(31,17,3,0xfffffff0,0x11223344);
    U32 result=first^second^Data;
    U32 words[8]={0x52323349,a,b,c,first,second,Data,result};
    emit(words,8);
    /* The API buffer itself belongs to the dependency's mapped data section. */
    emit(&Data,1);
    return result;
}
