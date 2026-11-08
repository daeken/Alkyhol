/* Original MIT observer for the actually measured default SECURITY_ATTRIBUTES.
 * The original lifecycle producer and its frozen bytes remain unchanged. */
#define MODE 0
#define entry original_unused_entry
#include "mutex-create.c"
#undef entry
struct Attributes {U length,descriptor,inherit;};
_Static_assert(sizeof(struct Attributes)==12,"Win32 SECURITY_ATTRIBUTES width");
U entry(void) {
    struct Attributes attributes={12,0,0};
#if CASE == 1
    attributes.inherit=1;
#elif CASE == 2
    attributes.descriptor=1;
#elif CASE == 3
    attributes.length=0;
#endif
    put(0x4d583332);put(sizeof(H));U before=stack();SetLastError(0xface1234);
    H first=CreateMutexW(&attributes,0,NAME);put(first!=0);put(GetLastError());if(!first)__builtin_trap();
    SetLastError(0xface1234);H second=CreateMutexW(0,0,NAME);put(second!=0);put(GetLastError());if(!second)__builtin_trap();
    SetLastError(0xface1234);put(CloseHandle(first)!=0);put(GetLastError());
    SetLastError(0xface1234);put(CloseHandle(second)!=0);put(GetLastError());
    SetLastError(0xface1234);H third=CreateMutexW(&attributes,0,NAME);put(third!=0);put(GetLastError());if(!third)__builtin_trap();
    SetLastError(0xface1234);put(CloseHandle(third)!=0);put(GetLastError());
    put(stack()==before);put(used+1);U done=0;
    if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
