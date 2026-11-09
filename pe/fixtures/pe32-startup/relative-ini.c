/* Original MIT: relative INI directory components reached by DAO's SKU query. */
typedef unsigned int U;typedef unsigned short W;typedef void* H;
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W*,U);
__declspec(dllimport) int __stdcall SetCurrentDirectoryW(const W*);
__declspec(dllimport) U __stdcall GetPrivateProfileStringW(const W*,const W*,const W*,W*,U,const W*);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U out[32],used;static void put(U n){out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U before=stack();put(0x52494e49);put(1);W path[1024];U count=GetModuleFileNameW(0,path,1024);if(!count||count>=1024)ExitProcess(2);while(count&&path[count-1]!='\\')--count;if(!count)ExitProcess(3);path[count-1]=0;put(SetCurrentDirectoryW(path)!=0);
 const W* names[]={L"owneddata\\settings.ini",L".\\owneddata\\settings.ini",L"owneddata\\absent.ini",L"absentdir\\settings.ini"};
 for(U i=0;i<4;i++){
  W buffer[34];for(U j=0;j<34;j++)buffer[j]=0x7171;SetLastError(0xface1234);
  U n=GetPrivateProfileStringW(L"Section",L"Key",L"fallback",buffer+1,32,names[i]);put(n);put(GetLastError());
  const W* expected=i<2?L"owned-value":L"fallback";U ok=n==(i<2?11u:8u);if(ok)for(U j=0;j<=n;j++)if(buffer[j+1]!=expected[j])ok=0;
  put(ok);put(buffer[0]==0x7171&&buffer[33]==0x7171);
 }
 put(stack()==before);put(used+1);U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
