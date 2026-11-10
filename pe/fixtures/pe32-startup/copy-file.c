/* Original MIT PE32 witness for the guarded copy import on DAO's load path. */
typedef unsigned int U;
typedef unsigned short W;
typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetEnvironmentVariableA(const char *,char *,U);
__declspec(dllimport) int __stdcall SetCurrentDirectoryW(const W *);
__declspec(dllimport) int __stdcall CopyFileW(const W *,const W *,int);
__declspec(dllimport) H __stdcall CreateFileW(const W *,U,U,H,U,U,H);
__declspec(dllimport) int __stdcall CloseHandle(H);
static void emit(const U *p,U n){U done=0;if(!WriteFile(GetStdHandle((U)-11),p,n*4,&done,0)||done!=n*4)__builtin_trap();}
static void name(W *out,const W *root,const char *leaf){U i=0;for(;root[i];++i)out[i]=root[i];out[i++]='\\';for(U j=0;;++j){out[i+j]=(W)leaf[j];if(!leaf[j])break;}}
U entry(void){
    char bytes[260];W root[260],source[260],target[260];
    U n=GetEnvironmentVariableA("ALKYHOL_COPY_ROOT",bytes,240);
    if(!n||n>=240)__builtin_trap();
    for(U i=0;i<=n;++i){if((unsigned char)bytes[i]>=128)__builtin_trap();root[i]=(W)(unsigned char)bytes[i];}
    if(!SetCurrentDirectoryW(root))__builtin_trap();
    name(source,root,MODE==3?"absent.bin":"source.bin");
    name(target,root,MODE==9?"..\\outside.bin":"target.bin");
    U head[2]={0x43503332,MODE};emit(head,2);
#if MODE == 8
    H held=CreateFileW(source,0x80000000,0,0,3,0x80,0);
    if(held==(H)(U)-1)__builtin_trap();
#endif
    SetLastError(0xaabbeeff);
    U result=(U)CopyFileW(MODE==7?(const W *)0:source,MODE==6?source:target,MODE==1||MODE==4||MODE==6||MODE==9?0:MODE==10?2:1);
    U words[2]={result,GetLastError()};emit(words,2);
#if MODE == 8
    if(!CloseHandle(held))__builtin_trap();
#endif
    return 0;
}
