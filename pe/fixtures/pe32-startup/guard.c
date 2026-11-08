/* Original MIT microtest for startup blockers; no game-derived code or data. */
typedef unsigned int U32;
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) void *__stdcall GetModuleHandleA(const char *);
__declspec(dllimport) void *__stdcall GetModuleHandleW(const unsigned short *);
__declspec(dllimport) void *__stdcall GetProcAddress(void *,const char *);
typedef U32 (__stdcall *AllocFls)(void (__stdcall *)(void *));
__declspec(dllimport) U32 __stdcall FlsAlloc(void (__stdcall *)(void *));
__declspec(dllimport) int __stdcall FlsFree(U32);
__declspec(dllimport) void *__stdcall FlsGetValue(U32);
__declspec(dllimport) int __stdcall FlsSetValue(U32,void *);
__declspec(dllimport) U32 __stdcall TlsAlloc(void);
__declspec(dllimport) int __stdcall TlsFree(U32);
__declspec(dllimport) void *__stdcall TlsGetValue(U32);
__declspec(dllimport) int __stdcall TlsSetValue(U32,void *);
__declspec(dllimport) void *__stdcall EncodePointer(void *);
__declspec(dllimport) void *__stdcall DecodePointer(void *);
__declspec(dllimport) int __stdcall InitializeCriticalSectionAndSpinCount(void *,U32);
__declspec(dllimport) void __stdcall InitializeCriticalSection(void *);
__declspec(dllimport) void __stdcall EnterCriticalSection(void *);
__declspec(dllimport) void __stdcall LeaveCriticalSection(void *);
__declspec(dllimport) void __stdcall DeleteCriticalSection(void *);
__declspec(dllimport) int __stdcall TryEnterCriticalSection(void *);
__declspec(dllimport) U32 __stdcall InterlockedIncrement(volatile U32 *);
__declspec(dllimport) U32 __stdcall GetCurrentThreadId(void);
struct Startup { U32 words[12];unsigned short show,reserved_size;void *reserved;void *handles[3]; };
_Static_assert(sizeof(struct Startup)==68,"Win32 STARTUPINFOA width");
__declspec(dllimport) void __stdcall GetStartupInfoA(struct Startup *);
__declspec(dllimport) U32 __stdcall GetFileType(void *);
__declspec(dllimport) U32 __stdcall SetHandleCount(U32);
__declspec(dllimport) void *__stdcall GetProcessHeap(void);
__declspec(dllimport) void *__stdcall HeapCreate(U32,U32,U32);
__declspec(dllimport) void *__stdcall HeapAlloc(void *,U32,U32);
__declspec(dllimport) int __stdcall HeapFree(void *,U32,void *);
struct Version {U32 size,major,minor,build,platform;char service[128];};
_Static_assert(sizeof(struct Version)==148,"Win32 OSVERSIONINFOA width");
__declspec(dllimport) int __stdcall GetVersionExA(struct Version *);
__declspec(dllimport) U32 __stdcall GetEnvironmentVariableA(const char *,char *,U32);
__declspec(dllimport) U32 __stdcall DeferredFunction(U32);
__declspec(dllimport) volatile U32 DeferredData;
static void emit(const U32 *data,U32 n) {
    U32 written=0;
    if(!WriteFile(GetStdHandle((U32)-11),data,n*4,&written,0)||written!=n*4) __builtin_trap();
}
#if MODE==6
static volatile U32 calls,received,reentered,tls_index;
static void __stdcall cleanup(void *value) {
    ++calls;received=(U32)value;
    reentered=(U32)TlsGetValue(tls_index);
}
#endif
U32 entry(void) {
#if MODE==10
    struct { struct Startup startup; U32 sentinel; } storage;
    storage.sentinel=0x1c2d3e4f;GetStartupInfoA(&storage.startup);
    struct Startup *s=&storage.startup;
    U32 words[12]={0x53493332,s->words[0]==68,(s->words[11]&0x100)!=0,
        s->reserved_size==0&&s->reserved==0,storage.sentinel==0x1c2d3e4f};
    for(U32 i=0;i<3;++i){words[5+i]=s->handles[i]==GetStdHandle((U32)-10-i);words[8+i]=GetFileType(s->handles[i])==3;}
    words[11]=GetFileType((void *)~0u)==0&&GetLastError()==6&&SetHandleCount(32)==32&&SetHandleCount(~0u)==~0u;emit(words,12);
#elif MODE==9
    U32 section[6],words[8]={0x43533332};
    words[1]=InitializeCriticalSectionAndSpinCount(section,4000)!=0;
    words[2]=TryEnterCriticalSection(section)!=0;EnterCriticalSection(section);
    LeaveCriticalSection(section);LeaveCriticalSection(section);
    words[3]=TryEnterCriticalSection(section)!=0;LeaveCriticalSection(section);
    DeleteCriticalSection(section);InitializeCriticalSection(section);
    EnterCriticalSection(section);LeaveCriticalSection(section);DeleteCriticalSection(section);
    volatile U32 value=0x7fffffff;
    words[4]=InterlockedIncrement(&value)==0x80000000&&value==0x80000000;
    value=~0u;words[5]=InterlockedIncrement(&value)==0&&value==0;
    U32 thread;__asm__("movl %%fs:0x24,%0":"=r"(thread));
    words[6]=thread!=0&&thread==GetCurrentThreadId()&&thread==GetCurrentThreadId();
    words[7]=sizeof(void *);emit(words,8);
#elif MODE==6
    void *kernel=GetModuleHandleA("KERNEL32.DLL");
    AllocFls allocate=(AllocFls)GetProcAddress(kernel,"FlsAlloc");
    U32 words[20]={0x4c533332,kernel!=0,GetModuleHandleA("kernel32")==kernel,
        GetModuleHandleW((const unsigned short *)L"kernel32.dll")==kernel,allocate!=0&&allocate==FlsAlloc};
    if(!allocate)__builtin_trap();
    tls_index=TlsAlloc();
    words[5]=tls_index!=~0u&&TlsGetValue(tls_index)==0;
    words[6]=TlsSetValue(tls_index,(void *)0xf1234567);
    words[7]=(U32)TlsGetValue(tls_index)==0xf1234567;
    U32 raw;__asm__("movl %%fs:0xe10(,%1,4),%0":"=r"(raw):"r"(tls_index));
    words[8]=raw==0xf1234567;
    U32 slot=allocate(cleanup);
    words[9]=slot!=~0u&&FlsGetValue(slot)==0;
    words[10]=FlsSetValue(slot,(void *)0x13572468);
    words[11]=(U32)FlsGetValue(slot)==0x13572468;
    words[12]=FlsFree(slot)&&calls==1&&received==0x13572468&&reentered==0xf1234567;
    slot=FlsAlloc(cleanup);words[13]=slot!=~0u&&FlsGetValue(slot)==0&&FlsFree(slot)&&calls==1;
    words[14]=TlsFree(tls_index)!=0;
    words[15]=TlsGetValue(~0u)==0&&GetLastError()==87;
    words[16]=FlsSetValue(~0u,0)==0&&GetLastError()==87;
    const U32 values[4]={0,1,0x80000000,~0u};words[17]=1;
    for(U32 i=0;i<4;++i) if((U32)DecodePointer(EncodePointer((void *)values[i]))!=values[i])words[17]=0;
    SetLastError(0x1234);(void)EncodePointer(0);words[18]=GetLastError()==0x1234;
    words[19]=sizeof(void *);emit(words,20);
#elif MODE==7 || MODE==8
    U32 boot=0x53543332;emit(&boot,1);
    void *kernel=GetModuleHandleA("kernel32.dll");
    (void)GetProcAddress(kernel,MODE==7?"UnsupportedStartupMicroExport":(const char *)1);
    emit(&boot,1); /* Unknown dynamic APIs/ordinals must stop, not look absent. */
#elif MODE==5
    char buffer[8];
    U32 needed=GetEnvironmentVariableA("ALKYHOL_STARTUP_MICRO",0,0);
    U32 small=GetEnvironmentVariableA("ALKYHOL_STARTUP_MICRO",buffer,2);
    U32 copied=GetEnvironmentVariableA("ALKYHOL_STARTUP_MICRO",buffer,sizeof(buffer));
    U32 contents=buffer[0]=='a'&&buffer[5]=='e'&&buffer[6]==0;
    U32 missing=GetEnvironmentVariableA("ALKYHOL_STARTUP_MICRO_ABSENT",buffer,sizeof(buffer));
    U32 words[8]={0x45333249,needed,small,copied,contents,missing,GetLastError(),sizeof(void *)};emit(words,8);
#elif MODE==4
    struct Version version;volatile unsigned char *bytes=(volatile unsigned char *)&version;
    for(U32 i=0;i<sizeof(version);++i)bytes[i]=0;
    version.size=sizeof(version);
    U32 ok=(U32)GetVersionExA(&version);
    U32 words[8]={0x56333249,ok,version.size,version.major,version.minor,version.build,version.platform,version.service[0]==0};
    emit(words,8);
#elif MODE==3
    void *heap=GetProcessHeap();void *private_heap=HeapCreate(0,4096,0);unsigned char *p=HeapAlloc(private_heap,8,37);
    if(!p)__builtin_trap();
    U32 words[8]={0x4833324d,heap!=0,heap==GetProcessHeap()&&private_heap!=heap,((U32)p&7)==0,p[0]==0&&p[36]==0,sizeof(void *),0,0};
    U32 *buffer=(U32 *)p;for(U32 i=0;i<6;++i)buffer[i]=words[i];emit(buffer,6);
    p[36]=255;words[6]=(U32)HeapFree(private_heap,0,p);
    unsigned char *q=HeapAlloc(private_heap,8,37);
    words[7]=q&&q[0]==0&&q[36]==0&&HeapFree(private_heap,0,q)&&HeapFree(private_heap,0,0);emit(words+6,2);
#else
    U32 boot=0x53543332;emit(&boot,1);
#if MODE==0
    U32 result=DeferredFunction(17);
#else
    U32 result=DeferredData;
#endif
    emit(&result,1); /* Must never run after either guarded import access. */
#endif
    return 0;
}
