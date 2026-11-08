/* Original MIT test of persistent scalar profile writes and paired reads. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W *,U);
__declspec(dllimport) int __stdcall WritePrivateProfileStringW(const W *,const W *,const W *,const W *);
__declspec(dllimport) U __stdcall GetPrivateProfileStringW(const W *,const W *,const W *,W *,U,const W *);
static W path[512];
static struct {U magic,pointers,step,wrote,write_error,count,read_error,balanced,before;W output[64];U after;} report;
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
    U n=GetModuleFileNameW(0,path,512);if(!n||n>=448)__builtin_trap();while(n&&path[n-1]!='\\')--n;if(!n)__builtin_trap();
    const W suffix[]=L"root\\alkyhol-owned-profile.ini";for(U i=0;i<sizeof(suffix)/sizeof(W);++i)path[n+i]=suffix[i];
    const W *sections[]={L"Section",L"Section",L"Section",L"Other",L"SECTION",L"Section",L"Section"};
    const W *keys[]={L"Key",L"Key",L"Empty",L"Flag",L"KEY",L"Second",L"Key"};
    const W *values[]={L"alpha",L"beta",L"",L"1",L"gamma",L"tail",L"final"};
    report.magic=0x50524632;report.pointers=sizeof(H);report.step=STEP;report.before=report.after=0x715e7a11;
    for(U i=0;i<64;++i)report.output[i]=0x7171;U before=stack();SetLastError(0xface1234);
    report.wrote=WritePrivateProfileStringW(sections[STEP],keys[STEP],values[STEP],path)!=0;report.write_error=GetLastError();
    SetLastError(0xface1234);report.count=GetPrivateProfileStringW(STEP==3?L"other":L"section",keys[STEP],L"fallback",report.output,64,path);report.read_error=GetLastError();report.balanced=stack()==before;
    U done=0;if(!WriteFile(GetStdHandle((U)-11),&report,sizeof(report),&done,0)||done!=sizeof(report))__builtin_trap();return 0;
}
