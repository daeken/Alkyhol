/* Original MIT owned-file test of real PE32 descriptors and Windows sharing. */
typedef unsigned int U;typedef int L;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetStdHandle(U);API U __stdcall GetModuleFileNameA(H,char *,U);API int __stdcall SetCurrentDirectoryA(const char *);
API H __stdcall CreateFileA(const char *,U,U,H,U,U,H);API int __stdcall CloseHandle(H);API U __stdcall GetLastError(void);
API int __stdcall WriteFile(H,const void *,U,U *,H);API int __stdcall ReadFile(H,void *,U,U *,H);API U __stdcall SetFilePointer(H,L,L *,U);
API int __stdcall SetFilePointerEx(H,long long,long long *,U);API int __stdcall SetEndOfFile(H);API int __stdcall FlushFileBuffers(H);
API int __stdcall GetFileInformationByHandle(H,void *);API U __stdcall GetFileSize(H,U *);
API int __stdcall GetHandleInformation(H,U*);
static U words[96],used;static char root[512];static void put(U v){words[used++]=v;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U n=GetModuleFileNameA(0,root,512);while(n&&root[n-1]!='\\')--n;const char tail[]="root";for(U i=0;i<sizeof(tail);++i)root[n+i]=tail[i];if(!SetCurrentDirectoryA(root))__builtin_trap();U before=stack();put(0x46494c32);put(5);
 H h=CreateFileA("owned.dat",0xc0000000,1,0,4,128,0);put(h!=(H)-1);put(GetLastError()==0);U done=0;put(WriteFile(h,"abcdef",6,&done,0)!=0&&done==6);put(FlushFileBuffers(h)!=0);put(GetFileSize(h,0)==6);
 H conflict=CreateFileA("owned.dat",0xc0000000,3,0,2,128,0);put(conflict==(H)-1);put(GetLastError()==32);put(GetFileSize(h,0)==6);
 put(SetFilePointer(h,-3,0,1)==3);char text[4]={0};put(ReadFile(h,text,3,&done,0)!=0&&done==3&&text[0]=='d'&&text[1]=='e'&&text[2]=='f');long long pos=-1;put(SetFilePointerEx(h,2,&pos,0)!=0&&pos==2);put(SetEndOfFile(h)!=0);put(GetFileSize(h,0)==2);U info[13];put(GetFileInformationByHandle(h,info)!=0);put(info[8]==0&&info[9]==2&&info[10]==1);put(CloseHandle(h)!=0);
 h=CreateFileA("owned.dat",0xc0000000,3,0,4,128,0);put(h!=(H)-1);put(GetLastError()==183);put(SetFilePointer(h,0,0,0)==0);text[0]=text[1]=0;put(ReadFile(h,text,4,&done,0)!=0&&done==2&&text[0]=='a'&&text[1]=='b');put(CloseHandle(h)!=0);
 h=CreateFileA("owned.dat",0x40000000,3,0,2,128,0);put(h!=(H)-1);put(GetLastError()==183);put(GetFileSize(h,0)==0);put(CloseHandle(h)!=0);struct {U size;H descriptor;int inherit;} attributes={12,0,1};
 h=CreateFileA("owned.dat",0x40000000,3,&attributes,3,128,0);put(h!=(H)-1);U flags=0;put(GetHandleInformation(h,&flags)!=0&&flags==1);put(CloseHandle(h)!=0);
 attributes.inherit=0;h=CreateFileA("owned.dat",0x40000000,3,&attributes,3,128,0);put(h!=(H)-1);flags=0xface1234;put(GetHandleInformation(h,&flags)!=0&&flags==0);put(CloseHandle(h)!=0);
 flags=0xface1234;put(GetHandleInformation(0,&flags)==0&&GetLastError()==6&&flags==0xface1234);
 put(stack()==before);put(used+1);if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;}
