/* Original MIT current-process identity and available-processor relationship. */
typedef unsigned int U;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetCurrentProcess(void);API U __stdcall GetCurrentProcessId(void);API U __stdcall GetProcessId(H);
API int __stdcall GetProcessAffinityMask(H,U *,U *);API void __stdcall GetSystemInfo(void *);API U __stdcall GetLastError(void);
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U before=stack(),p=0,s=0,info[9],out[12],used=0;H current=GetCurrentProcess();GetSystemInfo(info);out[used++]=0x50524332;out[used++]=4;out[used++]=(current==(H)-1);out[used++]=(GetProcessId(current)==GetCurrentProcessId());out[used++]=(GetProcessAffinityMask(current,&p,&s)!=0);out[used++]=(p&&s&&(p&s)==p&&s==info[4]);out[used++]=(GetProcessId((H)0)==0);out[used++]=GetLastError();out[used++]=(stack()==before);out[used++]=10;U done=0;if(!WriteFile(GetStdHandle((U)-11),out,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;}
