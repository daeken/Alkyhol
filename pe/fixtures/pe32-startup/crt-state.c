/* Original MIT microtest for CRT allocation/configuration reached by DAO DLLs.
 * No game input. The parent checks complete transport and native observations. */
typedef unsigned int U;typedef void *H;typedef void (__cdecl *Handler)(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __cdecl malloc(U);
__declspec(dllimport) int __cdecl _set_error_mode(int);
__declspec(dllimport) Handler __cdecl set_term(Handler) __asm("?set_terminate@@YAP6AXXZP6AXXZ@Z");
__declspec(dllimport) Handler __cdecl _get_terminate(void);
__declspec(dllimport) Handler __cdecl set_unexpected(Handler) __asm("?set_unexpected@@YAP6AXXZP6AXXZ@Z");
__declspec(dllimport) Handler __cdecl _get_unexpected(void);
__declspec(dllimport) void __cdecl __set_app_type(U);
__declspec(dllimport) U __cdecl __get_app_type(void);
static U words[128],used;
static void put(U n){if(used>=128)__builtin_trap();words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static volatile U fired;
static void one(void){fired=1;__builtin_trap();}
static void two(void){fired=2;__builtin_trap();}
U entry(void){
    put(0x43523332);put(sizeof(H));
    static const U modes[]={3,1,3,2,3,0,3};
    for(U i=0;i<7;i++){SetLastError(0xface1234);U before=stack();U old=(U)_set_error_mode((int)modes[i]);U after=stack();put(old);put(before==after);put(GetLastError());}
    SetLastError(0xface1234);Handler initial=_get_terminate();put(initial==0);
    U before=stack();Handler old=set_term(one);U after=stack();put(old==initial);put(before==after);put(_get_terminate()==one);
    old=set_term(two);put(old==one);put(_get_terminate()==two);old=set_term(initial);put(old==two);put(_get_terminate()==initial);put(GetLastError());
    SetLastError(0xface1234);initial=_get_unexpected();put(initial==0);
    before=stack();old=set_unexpected(one);after=stack();put(old==initial);put(before==after);put(_get_unexpected()==one);
    old=set_unexpected(two);put(old==one);put(_get_unexpected()==two);old=set_unexpected(initial);put(old==two);put(_get_unexpected()==initial);put(GetLastError());
    static const U sizes[]={0,1,17,4099,65539};unsigned char *blocks[5];
    for(U i=0;i<5;i++){
        SetLastError(0xface1234);before=stack();blocks[i]=(unsigned char *)malloc(sizes[i]);after=stack();put(blocks[i]!=0);put((U)blocks[i]&7);put(before==after);put(GetLastError());
        if(!blocks[i])__builtin_trap();for(U j=0;j<sizes[i];j++)blocks[i][j]=(unsigned char)(j*17+i);
    }
    for(U i=0;i<5;i++){U sum=0;for(U j=0;j<sizes[i];j++){if(blocks[i][j]!=(unsigned char)(j*17+i))__builtin_trap();sum+=blocks[i][j];}put(sum);}
    SetLastError(0xface1234);U initial_type=__get_app_type();put(initial_type);
    before=stack();__set_app_type(1);after=stack();put(before==after);put(__get_app_type());
    __set_app_type(2);put(__get_app_type());__set_app_type(initial_type);put(__get_app_type()==initial_type);put(GetLastError());
    put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
