/* Alkyhol MIT: ordinary DeleteFile ABI, errors and profile exclusions. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API U __stdcall GetEnvironmentVariableA(const char *,char *,U);API int __stdcall SetCurrentDirectoryW(const W *);
API int __stdcall DeleteFileW(const W *);API int __stdcall DeleteFileA(const char *);
API H __stdcall CreateFileW(const W *,U,U,H,U,U,H);API int __stdcall CloseHandle(H);
API U __stdcall GetLastError(void);API void __stdcall SetLastError(U);
static void emit(U *data,U count){U done=0;if(!WriteFile(GetStdHandle((U)-11),data,count*4,&done,0)||done!=count*4)__builtin_trap();}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){char bytes[260],mode[8];W root[260],path[64];U n=GetEnvironmentVariableA("ALKYHOL_DELETE_ROOT",bytes,240);if(!n||n>=240)__builtin_trap();
 for(U i=0;i<=n;++i)root[i]=(W)(unsigned char)bytes[i];if(!SetCurrentDirectoryW(root))__builtin_trap();
 if(!GetEnvironmentVariableA("ALKYHOL_DELETE_MODE",mode,8))__builtin_trap();U m=0;for(U i=0;mode[i];++i)m=m*10+(U)(mode[i]-'0');
 const char *name=m==1?"missing.bin":m==2?"missing\\target.bin":m==4?"folder":m==8?"..\\outside.bin":"target.bin";
 for(U i=0;;++i){path[i]=(W)(unsigned char)name[i];if(!name[i])break;}
 H held=0;if(m==5||m==7){held=CreateFileW(path,0x80000000,m==7?5:1,0,3,0x80,0);if(held==(H)(U)-1)__builtin_trap();}
 U head[2]={0x444c3332,m};emit(head,2);U before=stack();SetLastError(0xaabbeeff);
 U result=m==6?(U)DeleteFileA(name):(U)DeleteFileW(m==11?(const W *)0:path);U error=GetLastError(),repeat=0,repeat_error=0;
 if(result){repeat=(U)DeleteFileW(path);repeat_error=GetLastError();}
 U tail[6]={result,error,repeat,repeat_error,stack()==before,held?(U)CloseHandle(held):1};emit(tail,6);return 0;
}
