/* Original MIT microtest for startup blockers; no game-derived code or data. */
typedef unsigned int U32;
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
__declspec(dllimport) U32 __stdcall GetLastError(void);
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
U32 entry(void) {
#if MODE==5
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
