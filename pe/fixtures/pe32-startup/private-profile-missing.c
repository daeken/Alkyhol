/* Original MIT observer for DAO's reached INI-read API, owned absent paths. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W *,U);
__declspec(dllimport) int __stdcall SetCurrentDirectoryW(const W *);
__declspec(dllimport) U __stdcall GetPrivateProfileStringW(const W *,const W *,const W *,W *,U,const W *);
static struct {U index,capacity,result,error,balanced,before;W output[32];U after;} rows[10];
static W path[512];static U prefix;
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static void filename(const W *text){U i=0;do{path[prefix+i]=text[i];}while(text[i++]);}
U entry(void){
    (void)rows;
    U n=GetModuleFileNameW(0,path,512);if(!n||n>=512)__builtin_trap();prefix=n;while(prefix&&path[prefix-1]!='\\')--prefix;if(!prefix||prefix>448)__builtin_trap();
    path[prefix-1]=0;if(!SetCurrentDirectoryW(path))__builtin_trap();path[prefix-1]='\\';
    filename(L"alkyhol-absent-profile.ini");
#if MODE == 1
    (void)GetPrivateProfileStringW(0,L"Key",L"fallback",rows[0].output,32,path);__builtin_trap();
#elif MODE == 2
    (void)GetPrivateProfileStringW(L"Section",0,L"fallback",rows[0].output,32,path);__builtin_trap();
#elif MODE == 3
    (void)GetPrivateProfileStringW(L"Section",L"Key",L"fallback",rows[0].output,0,path);__builtin_trap();
#elif MODE == 4
    (void)GetPrivateProfileStringW(L"Section",L"Key",L"fallback",(W *)1,32,path);__builtin_trap();
#elif MODE == 5
    (void)GetPrivateProfileStringW(L"Section",L"Key",L"fallback",rows[0].output,32,L"alkyhol-absent-profile.ini");__builtin_trap();
#elif MODE == 6
    filename(L"alkyhol-owned-profile.ini");(void)GetPrivateProfileStringW(L"Section",L"Key",L"fallback",rows[0].output,32,path);__builtin_trap();
#elif MODE == 7
    (void)GetPrivateProfileStringW(L"Section",L"Key",L"fallback\t ",rows[0].output,32,path);__builtin_trap();
#else
    const W *defaults[]={L"fallback",0,L"  fallback   ",L"",L"π桜🌸",L"abcdef",L"abcdef",L"fallback",L"   ",L"abcdef"};
    const U sizes[]={32,32,32,32,32,4,1,32,32,7};
    for(U i=0;i<10;++i){
        filename(i==7?L"absent-profile-parent\\settings.ini":L"alkyhol-absent-profile.ini");
        rows[i].index=i;rows[i].capacity=sizes[i];rows[i].before=rows[i].after=0x715e7a11;
        for(U k=0;k<32;++k)rows[i].output[k]=0x7171;
        U before=stack();SetLastError(0xface1234);
        rows[i].result=GetPrivateProfileStringW(L"Section",L"Key",defaults[i],rows[i].output,sizes[i],i%2==0?L".\\alkyhol-absent-profile.ini":path);
        rows[i].error=GetLastError();rows[i].balanced=stack()==before;
    }
    U done=0;const U header[]={0x494e4932,4,10,92};
    if(!WriteFile(GetStdHandle((U)-11),header,sizeof(header),&done,0)||done!=sizeof(header))__builtin_trap();
    if(!WriteFile(GetStdHandle((U)-11),rows,sizeof(rows),&done,0)||done!=sizeof(rows))__builtin_trap();return 0;
#endif
}
