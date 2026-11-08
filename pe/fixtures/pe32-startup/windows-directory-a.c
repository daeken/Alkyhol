/* Original MIT witness for Bink's reached complete-buffer directory query. */
typedef unsigned int U;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetWindowsDirectoryA(char *,U);
static struct {U before;char text[1024];U after;} guarded __attribute__((unused));
static __attribute__((unused)) U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
#if MODE == 1
    (void)GetWindowsDirectoryA((char *)1,1024);__builtin_trap();
#elif MODE == 2
    (void)GetWindowsDirectoryA(guarded.text,1);__builtin_trap();
#else
    guarded.before=guarded.after=0x715e7a11;for(U i=0;i<1024;i++)guarded.text[i]=(char)0xa5;
    SetLastError(0xface1234);U before=stack(),n=GetWindowsDirectoryA(guarded.text,1024),after=stack(),error=GetLastError();if(!n||n>=1023)__builtin_trap();
    U words[10]={0x57443332,sizeof(H),n,before==after,guarded.before,guarded.after,guarded.text[n]==0,(unsigned char)guarded.text[n+1]==0xa5,error,10},done=0;
    H out=GetStdHandle((U)-11);if(!WriteFile(out,words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();
    if(!WriteFile(out,guarded.text,n+1,&done,0)||done!=n+1)__builtin_trap();return 0;
#endif
}
