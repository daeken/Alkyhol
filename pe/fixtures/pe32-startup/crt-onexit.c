/* Original MIT witness for reached DLL exit-table/lock and wide startup APIs. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;typedef int (__cdecl *Fn)(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) void __cdecl _lock(int);
__declspec(dllimport) void __cdecl _unlock(int);
__declspec(dllimport) Fn __cdecl __dllonexit(Fn,Fn **,Fn **);
__declspec(dllimport) H __cdecl malloc(U);
struct Info {U size;H reserved,desktop,title;U x,y,width,height,columns,rows,fill,flags;W show,bytes;H data,in,out,error;};
_Static_assert(sizeof(struct Info)==68,"Win32 STARTUPINFO width");
__declspec(dllimport) void __stdcall GetStartupInfoA(struct Info *);
__declspec(dllimport) void __stdcall GetStartupInfoW(struct Info *);
__declspec(dllimport) U __stdcall GetModuleFileNameA(H,char *,U);
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W *,U);
static U words[80],used,counter;
void *memset(void *p,int value,U size){volatile unsigned char *q=p;for(U i=0;i<size;i++)q[i]=(unsigned char)value;return p;}
static void put(U n){if(used>=80)__builtin_trap();words[used++]=n;}
static __attribute__((unused)) U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
int one(void){counter=counter*10+1;return 10;}
int two(void){counter=counter*10+2;return 20;}
struct Table {U before;Fn *begin,*end;U after;};
U entry(void){
    put(0x4f583332);put(sizeof(H));
    struct Table a={0x715e7a11,0,0,0x715e7a11};(void)a;
#if MODE == 1
    _unlock(8);
#elif MODE == 2
    _lock(7);
#elif MODE == 3
    (void)__dllonexit(0,&a.begin,&a.end);
#elif MODE == 4
    (void)__dllonexit(one,(Fn **)1,&a.end);
#else
    SetLastError(0xface1234);U before=stack();_lock(8);_lock(8);_unlock(8);_unlock(8);U after=stack();put(before==after);put(GetLastError());
    _lock(8);Fn ret=__dllonexit(one,&a.begin,&a.end);
    put(ret==one);put(a.begin!=0);put((U)a.begin&7);put((U)(a.end-a.begin));put(a.begin[0]==one);put(GetLastError());
    ret=__dllonexit(two,&a.begin,&a.end);
    put(ret==two);put((U)(a.end-a.begin));put(a.begin[0]==one);put(a.begin[1]==two);put(counter==0);put(a.before==0x715e7a11&&a.after==0x715e7a11);put(GetLastError());
    before=stack();ret=__dllonexit(one,&a.begin,&a.end);after=stack();
    put(ret==one);put((U)(a.end-a.begin));put(a.begin[0]==one);put(a.begin[1]==two);put(a.begin[2]==one);put(a.before==0x715e7a11&&a.after==0x715e7a11);put(before==after);_unlock(8);
    counter=0;for(Fn *p=a.end;p!=a.begin;){--p;put((U)(*p)());}put(counter);
    struct Table b={0x715e7a11,(Fn *)malloc(128),0,0x715e7a11};put(b.begin!=0);if(!b.begin)__builtin_trap();
    b.begin[0]=one;b.end=b.begin+1;_lock(8);ret=__dllonexit(two,&b.begin,&b.end);_unlock(8);
    put(ret==two);put((U)(b.end-b.begin));put(b.begin[0]==one);put(b.begin[1]==two);put(b.before==0x715e7a11&&b.after==0x715e7a11);
    counter=0;put(b.end[-1]()==20&&b.begin[0]()==10);put(counter);put(GetLastError());
    struct {U before;struct Info value;U after;} narrow={0},wide={0};narrow.before=narrow.after=wide.before=wide.after=0x715e7a11;
    SetLastError(0xface1234);before=stack();GetStartupInfoA(&narrow.value);GetStartupInfoW(&wide.value);after=stack();
    put(before==after);put(narrow.value.size);put(wide.value.size);put(narrow.value.flags==wide.value.flags);put((wide.value.flags&0x100)!=0);
    put(wide.value.in==GetStdHandle((U)-10));put(wide.value.out==GetStdHandle((U)-11));put(wide.value.error==GetStdHandle((U)-12));
    put(narrow.before==0x715e7a11&&narrow.after==0x715e7a11&&wide.before==0x715e7a11&&wide.after==0x715e7a11);put(GetLastError());
    struct {U before;char text[1024];U after;} path_a={0};struct {U before;W text[1024];U after;} path_w={0};path_a.before=path_a.after=path_w.before=path_w.after=0x715e7a11;
    SetLastError(0xface1234);U na=GetModuleFileNameA(0,path_a.text,1024);put(GetLastError());SetLastError(0xface1234);U nw=GetModuleFileNameW(0,path_w.text,1024);put(na>0&&na==nw&&na<1024);
    U same=1;for(U i=0;i<=na&&i<1024;i++)if((unsigned char)path_a.text[i]!=path_w.text[i])same=0;put(same);
    put(path_a.before==0x715e7a11&&path_a.after==0x715e7a11&&path_w.before==0x715e7a11&&path_w.after==0x715e7a11);put(GetLastError());
    put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
    __builtin_trap();
}
