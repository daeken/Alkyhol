/* Original MIT owned PE32 dynamic-library lifecycle/calling-convention observer. */
typedef unsigned int U32;
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
extern char __ImageBase;
static U32 calls;
static U32 self;
U32 Data=0x31415926;
static void emit(U32 *words,U32 count) {
    U32 written=0;
    if(!WriteFile(GetStdHandle((U32)-11),words,count*4,&written,0)||written!=count*4)
        __builtin_trap();
}
int __stdcall DllMain(void *module,U32 reason,void *reserved) {
    U32 fs_self,limit,base;
    __asm__ volatile("movl %%fs:0x18,%0":"=r"(fs_self));
    __asm__ volatile("movl %%fs:8,%0":"=r"(limit));
    __asm__ volatile("movl %%fs:4,%0":"=r"(base));
    SetLastError(0x12345678);
    U32 words[12]={0x4c323349,REJECT,reason,reserved==0,
        module==&__ImageBase,sizeof(void *),fs_self!=0,
        (U32)&words>=limit&&(U32)&words<base,GetLastError(),calls,Data,0};
    words[11]=reason==1||fs_self==self;
    if(reason==1) self=fs_self;
    emit(words,12);
    return !REJECT;
}
U32 __stdcall RunStdcall(U32 a,U32 b,U32 c,U32 d,U32 e) {
    U32 fs_self;
    __asm__ volatile("movl %%fs:0x18,%0":"=r"(fs_self));
    ++calls;
    SetLastError(0x88776655);
    U32 result=(a+3*b)^(c+5*d)^e;
    U32 words[8]={0x53323349,5,result,calls,fs_self==self,GetLastError(),Data,0xa1b2c3d4};
    emit(words,8);
    return result;
}
U32 __cdecl RunCdecl(U32 a,U32 b,U32 c) {
    ++calls;
    U32 result=(a^b)+c;
    U32 words[8]={0x43323349,3,result,calls,Data,0x11223344,0x55667788,0x99aabbcc};
    emit(words,8);
    return result;
}
