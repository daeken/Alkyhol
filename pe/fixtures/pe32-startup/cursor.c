/* Original MIT read-only cursor ABI witness. Never moves the user's mouse. */
typedef unsigned int U;typedef void *H;
struct Point {int x,y;};
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) int __stdcall GetCursorPos(struct Point *);
static __attribute__((unused)) U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
#if MODE == 1
    (void)GetCursorPos((struct Point *)1);__builtin_trap();
#else
    struct {U before;struct Point point;U after;} guarded={0x715e7a11,{(int)0x80000000,(int)0x80000000},0x715e7a11};
    SetLastError(0xface1234);U before=stack();int result=GetCursorPos(&guarded.point);U after=stack(),error=GetLastError();
    U words[11]={0x43503332,sizeof(H),sizeof(struct Point),(U)result,before==after,guarded.before,guarded.after,error,(U)guarded.point.x,(U)guarded.point.y,11};
    U done=0;if(!WriteFile(GetStdHandle((U)-11),words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();return 0;
#endif
}
