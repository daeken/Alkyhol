/* Original MIT microtest: real cdecl exception-policy callbacks and exit gates. */
typedef unsigned int U;typedef void (__cdecl *Handler)(void);
__declspec(dllimport) void __stdcall ExitProcess(U);
__declspec(dllimport) Handler __cdecl set_term(Handler) __asm("?set_terminate@@YAP6AXXZP6AXXZ@Z");
__declspec(dllimport) Handler __cdecl set_unexpected(Handler) __asm("?set_unexpected@@YAP6AXXZP6AXXZ@Z");
__declspec(dllimport) void __cdecl terminate(void) __asm("?terminate@@YAXXZ");
__declspec(dllimport) void __cdecl unexpected(void) __asm("?unexpected@@YAXXZ");
__declspec(dllimport) void * __cdecl malloc(U);
__declspec(dllimport) int __cdecl _set_error_mode(int);
__declspec(dllimport) void __cdecl __set_app_type(U);
void term_exit(void){ExitProcess(37);__builtin_trap();}
void unexpected_exit(void){ExitProcess(38);__builtin_trap();}
void returning(void){}
U entry(void){
#if MODE == 1
    terminate();
#elif MODE == 2
    set_term(term_exit);terminate();
#elif MODE == 3
    set_term(returning);terminate();
#elif MODE == 4
    set_unexpected(unexpected_exit);unexpected();
#elif MODE == 5
    set_term(term_exit);set_unexpected(returning);unexpected();
#elif MODE == 6
    set_term((Handler)1);
#elif MODE == 7
    (void)malloc(0xffffffff);
#elif MODE == 8
    (void)_set_error_mode(-1);
#elif MODE == 9
    __set_app_type(4);
#endif
    __builtin_trap();
}
