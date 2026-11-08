/* Original MIT targeted bridge test for DAO's activation lookup. Only a
 * verified empty embedded context is supported; unknown contexts must stop. */
typedef unsigned int U32;
typedef unsigned short W;
typedef void *H;
typedef int (__stdcall *Find)(U32,const void *,U32,const W *,void *);
__declspec(dllimport) H __stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U32,U32 *,H);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) H __stdcall GetModuleHandleW(const W *);
__declspec(dllimport) H __stdcall GetProcAddress(H,const char *);
__declspec(dllimport) int __stdcall FindActCtxSectionStringW(U32,const void *,U32,const W *,void *);
#if MODE==4
__declspec(dllimport) U32 __stdcall OwnedDependency(void);
#endif
static void emit(const U32 *data,U32 count) {
    U32 written=0;if(!WriteFile(GetStdHandle((U32)-11),data,count*4,&written,0)||written!=count*4)__builtin_trap();
}
U32 entry(void) {
    U32 marker=0x414d3332;emit(&marker,1);
    const W module[]={ 'k','e','r','n','e','l','3','2',0 };
    const W key[]={ 'o','w','n','e','d','-','m','i','s','s','i','n','g','.','d','l','l',0 };
    Find find=(Find)GetProcAddress(GetModuleHandleW(module),"FindActCtxSectionStringW");
    if(!find)__builtin_trap();
#if MODE==4
    if(OwnedDependency()!=1)__builtin_trap();
#endif
#if MODE==0
    for(U32 flags=0;flags<2;flags++)for(U32 size=40;size<=64;size+=24) {
        U32 data[17];for(U32 i=0;i<17;i++)data[i]=0x715e7a11;data[0]=size;
        SetLastError(0xface1234);U32 result=(U32)find(flags,0,2,key,data),error=GetLastError();
        U32 unchanged=data[0]==size;for(U32 i=1;i<17;i++)if(data[i]!=0x715e7a11)unchanged=0;
        U32 words[8]={size,flags,result,error,unchanged,find==FindActCtxSectionStringW,sizeof(void *),data[16]};emit(words,8);
    }
#else
    U32 data[17];for(U32 i=0;i<17;i++)data[i]=0x715e7a11;data[0]=64;
#if MODE==2
    static const U32 readonly[16]={64};
    (void)find(0,0,2,key,(void *)readonly);
#elif MODE==3
    (void)find(2,0,2,key,data);
#else
    (void)find(0,0,2,key,data);
#endif
    emit(&marker,1); /* Unknown/unsupported forms must not return here. */
#endif
    return 0;
}
