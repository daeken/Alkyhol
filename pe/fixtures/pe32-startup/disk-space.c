/* Original MIT: DAO's real filesystem-space query. Values vary by machine;
 * compare API/guard/error laws, never manufacture matching disk capacities. */
typedef unsigned int U;typedef unsigned long long Q;typedef void* H;
__declspec(dllimport) int __stdcall GetDiskFreeSpaceExA(const char*,Q*,Q*,Q*);
__declspec(dllimport) U __stdcall GetModuleFileNameA(H,char*,U);
__declspec(dllimport) int __stdcall SetCurrentDirectoryA(const char*);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U out[32],used;static void put(U n){out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U before=stack();put(0x4b534944);put(1);char path[1024];U n=GetModuleFileNameA(0,path,1024);if(!n||n>=1024)ExitProcess(2);while(n&&path[n-1]!='\\')--n;if(!n)ExitProcess(3);path[n-1]=0;put(SetCurrentDirectoryA(path)!=0);
 Q values[5]={0x715e7a112344bbccULL,0,0,0,0x715e7a112344bbccULL};
 put(GetDiskFreeSpaceExA(0,values+1,values+2,values+3)!=0);put(values[0]==0x715e7a112344bbccULL&&values[4]==0x715e7a112344bbccULL);put(values[2]>0&&values[1]<=values[2]&&values[3]<=values[2]);
 Q single[3]={0x12345678abcddcbaULL,0,0x12345678abcddcbaULL};put(GetDiskFreeSpaceExA(path,single+1,0,0)!=0);put(single[1]>0);put(single[0]==0x12345678abcddcbaULL&&single[2]==0x12345678abcddcbaULL);
 path[n-1]='\\';const char suffix[]="owned-disk-absent\\child";for(U i=0;i<sizeof(suffix);i++)path[n+i]=suffix[i];values[1]=values[2]=values[3]=0x1111222233334444ULL;
 put(GetDiskFreeSpaceExA(path,values+1,values+2,values+3)==0);put(GetLastError());put(values[0]==0x715e7a112344bbccULL&&values[4]==0x715e7a112344bbccULL&&values[1]==0x1111222233334444ULL&&values[2]==values[1]&&values[3]==values[1]);
 put(stack()==before);put(used+1);U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
