/* Original MIT observer: actual current-user Documents query reached by DAO. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) H __stdcall GetModuleHandleW(const W *);
__declspec(dllimport) H __stdcall GetProcAddress(H,const char *);
__declspec(dllimport) U __stdcall SHGetFolderPathW(H,int,H,U,W *);
typedef U (__stdcall *Query)(H,int,H,U,W *);
struct Buffer {U before;W path[260];U after;};
static struct {U magic,pointers,hr[2],gle[2],balanced,length[2];struct Buffer data[2];U end;} report;
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
    report.magic=0x444f4332;report.pointers=sizeof(H);report.end=0x656e6432;
    for(U j=0;j<2;++j){report.data[j].before=report.data[j].after=0x715e7a11;for(U i=0;i<260;++i)report.data[j].path[i]=0x7171;}
    U before=stack();(void)before;
#if MODE == 1
    (void)SHGetFolderPathW(0,26,0,0,report.data[0].path);__builtin_trap();
#elif MODE == 2
    (void)SHGetFolderPathW(0,5,(H)-1,0,report.data[0].path);__builtin_trap();
#elif MODE == 3
    (void)SHGetFolderPathW(0,5,0,1,report.data[0].path);__builtin_trap();
#elif MODE == 4
    (void)SHGetFolderPathW(0,5,0,0,(W *)1);__builtin_trap();
#elif MODE == 5
    (void)SHGetFolderPathW((H)1,5,0,0,report.data[0].path);__builtin_trap();
#else
    SetLastError(0xface1234);report.hr[0]=SHGetFolderPathW(0,5,0,0,report.data[0].path);report.gle[0]=GetLastError();
    Query query=(Query)GetProcAddress(GetModuleHandleW(L"shell32.dll"),"SHGetFolderPathW");if(!query)__builtin_trap();
    SetLastError(0xface1234);report.hr[1]=query(0,5,0,0,report.data[1].path);report.gle[1]=GetLastError();
    report.balanced=stack()==before;
    for(U j=0;j<2;++j){while(report.length[j]<260&&report.data[j].path[report.length[j]])++report.length[j];}
    U done=0;if(!WriteFile(GetStdHandle((U)-11),&report,sizeof(report),&done,0)||done!=sizeof(report))__builtin_trap();return 0;
#endif
}
