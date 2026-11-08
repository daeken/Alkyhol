/* Original MIT registration witness. It does not raise or recover an exception. */
typedef unsigned int U;typedef void *H;typedef int (__stdcall *Filter)(H);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) Filter __stdcall SetUnhandledExceptionFilter(Filter);
static volatile U callbacks;
int __stdcall first(H argument){(void)argument;callbacks++;return -1;}
int __stdcall second(H argument){(void)argument;callbacks++;return 0;}
static __attribute__((unused)) U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
#if MODE == 1
    (void)SetUnhandledExceptionFilter((Filter)1);__builtin_trap();
#else
    U words[11]={0x45523332,sizeof(H)};SetLastError(0xface1234);U before=stack();
    words[2]=SetUnhandledExceptionFilter(first)==0;
    words[3]=SetUnhandledExceptionFilter(first)==first;
    words[4]=SetUnhandledExceptionFilter(second)==first;
    words[5]=SetUnhandledExceptionFilter(0)==second;
    words[6]=SetUnhandledExceptionFilter(0)==0;
    words[7]=callbacks==0;U after=stack();words[8]=before==after;words[9]=GetLastError();words[10]=11;
    U done=0;if(!WriteFile(GetStdHandle((U)-11),words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();return 0;
#endif
}
