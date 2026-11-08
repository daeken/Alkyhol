/* Original MIT observer for the GetSystemInfo call reached by DAO startup. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
struct Info {W architecture,reserved;U page;H minimum,maximum;U mask,count,type,granularity;W level,revision;};
_Static_assert(sizeof(struct Info)==36,"Win32 SYSTEM_INFO width");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) void __stdcall GetSystemInfo(struct Info *);
static struct {U before;struct Info value;U after;} guarded __attribute__((unused));
static U words[25] __attribute__((unused));
static __attribute__((unused)) U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
#if MODE == 1
    GetSystemInfo((struct Info *)1);__builtin_trap();
#else
    guarded.before=guarded.after=0x715e7a11;unsigned char *bytes=(unsigned char *)&guarded.value;
    for(U i=0;i<36;i++)bytes[i]=0xa5;
    SetLastError(0xface1234);U before=stack();GetSystemInfo(&guarded.value);U after=stack(),error=GetLastError();
    words[0]=0x53493332;words[1]=sizeof(H);words[2]=sizeof(struct Info);words[3]=before==after;words[4]=guarded.before;words[5]=guarded.after;words[6]=error;words[7]=25;
    volatile unsigned char *destination=(volatile unsigned char *)&words[8];for(U i=0;i<36;i++)destination[i]=bytes[i];
    for(U leaf=0;leaf<2;leaf++){U a,b,c,d;__asm__ volatile("cpuid":"=a"(a),"=b"(b),"=c"(c),"=d"(d):"0"(leaf),"2"(0));U at=17+leaf*4;words[at]=a;words[at+1]=b;words[at+2]=c;words[at+3]=d;}
    U done=0;if(!WriteFile(GetStdHandle((U)-11),words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();return 0;
#endif
}
