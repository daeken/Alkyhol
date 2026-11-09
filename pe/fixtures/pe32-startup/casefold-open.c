/* Original MIT: DAO lowercases complete settings paths before CreateFileA. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) U __stdcall GetModuleFileNameA(H,char*,U);
__declspec(dllimport) int __stdcall SetCurrentDirectoryA(const char*);
__declspec(dllimport) H __stdcall CreateFileA(const char*,U,U,H,U,U,H);
__declspec(dllimport) int __stdcall ReadFile(H,void*,U,U*,H);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U out[24],used;static void put(U n){out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U before=stack();put(0x464c5343);put(1);char path[1024];U n=GetModuleFileNameA(0,path,1024);if(!n||n>=1024)ExitProcess(2);while(n&&path[n-1]!='\\')--n;if(!n)ExitProcess(3);path[n-1]=0;if(!SetCurrentDirectoryA(path))ExitProcess(3);path[n-1]='\\';
 const char suffix[]="OwnedCase\\MiXeD.DAT";for(U i=0;i<sizeof(suffix);i++)path[n+i]=suffix[i];
 H first=CreateFileA(path,0x80000000,1,0,3,0x80,0);put(first!=(H)-1);if(first==(H)-1){U error=GetLastError(),done=0,bytes=0;while(path[bytes])++bytes;WriteFile(GetStdHandle((U)-12),path,bytes,&done,0);put(error);WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(4);}
 for(U i=0;path[i];i++)if(path[i]>='A'&&path[i]<='Z')path[i]+='a'-'A';
 H second=CreateFileA(path,0x80000000,1,0,3,0x80,0);put(second!=(H)-1);if(second==(H)-1)ExitProcess(5);
 U value=0,bytes=0;put(ReadFile(second,&value,4,&bytes,0)&&bytes==4&&value==0x65736163);put(CloseHandle(second));put(CloseHandle(first));
 const char missing[]="ownedcase\\missing.dat";for(U i=0;i<sizeof(missing);i++)path[n+i]=missing[i];put(CreateFileA(path,0x80000000,1,0,3,0x80,0)==(H)-1);put(GetLastError());
 const char parent[]="missingparent\\leaf.dat";for(U i=0;i<sizeof(parent);i++)path[n+i]=parent[i];put(CreateFileA(path,0x80000000,1,0,3,0x80,0)==(H)-1);put(GetLastError());
 put(stack()==before);put(used+1);U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
