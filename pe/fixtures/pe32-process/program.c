/* Original MIT PE32 static process graph observer, not game-derived code. */
typedef unsigned int U32;
typedef unsigned short U16;
__declspec(dllimport) void *__stdcall GetModuleHandleA(const char *);
__declspec(dllimport) void *__stdcall GetModuleHandleW(const U16 *);
__declspec(dllimport) void *__stdcall GetProcAddress(void *,const char *);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
__declspec(dllimport) U32 __cdecl Run(U32,U32,U32);
__declspec(dllimport) U32 __cdecl RunCdecl(U32,U32,U32);
__declspec(dllimport) U32 Data;
__declspec(dllimport) void __stdcall Finish(U32);
extern char __ImageBase;
static void emit(U32 *words,U32 count) {
    U32 written=0;
    if(!WriteFile(GetStdHandle((U32)-11),words,count*4,&written,0)||written!=count*4) __builtin_trap();
}
U32 entry(void) {
    const U16 accepted[]={ 'A','C','C','E','P','T','E','D','.','D','L','L',0 };
    SetLastError(0x13572468);
    void *self_a=GetModuleHandleA(0), *self_w=GetModuleHandleW(0);
    void *leaf=GetModuleHandleA("AcCePtEd"), *wide=GetModuleHandleW(accepted);
    void *consumer=GetModuleHandleA("consumer.dll");
    U32 module_error=GetLastError();
    void *data=GetProcAddress(leaf,"Data");
    void *code=GetProcAddress(leaf,"RunCdecl");
    void *ordinal=GetProcAddress(leaf,(const char *)2);
    void *run=GetProcAddress(consumer,"Run");
    U32 symbol_error=GetLastError();
    U32 absent_module=GetModuleHandleA("missing-owned.dll")==0;
    U32 missing_module_error=GetLastError();
    U32 absent_name=GetProcAddress(leaf,"MissingOwnedSymbol")==0;
    U32 missing_name_error=GetLastError();
    U32 absent_ordinal=GetProcAddress(leaf,(const char *)65535)==0;
    U32 missing_ordinal_error=GetLastError();
    U32 words[24]={0x50333245,self_a==&__ImageBase,self_w==self_a,
        leaf!=0,leaf==wide,consumer!=0,consumer!=leaf,data==&Data,
        code==(void *)RunCdecl,ordinal==code,run==(void *)Run,
        module_error,symbol_error,absent_module,missing_module_error,
        absent_name,missing_name_error,absent_ordinal,missing_ordinal_error,
        Data,sizeof(void *),0,0,0x10203040};
    U32 (*call)(U32,U32,U32)=(U32 (*)(U32,U32,U32))run;
    words[21]=call(0xabcdef01,0x12345678,19);
    words[22]=Data;
    emit(words,24);
    /* A real imported DLL function, not an EXE-side synthetic exit request. */
    Finish(7);
    /* Neither this output nor a return may occur after ExitProcess. */
    words[0]=0xbadbad00;emit(words,24);
    return 99;
}
