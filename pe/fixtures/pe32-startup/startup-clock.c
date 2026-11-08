/* Original MIT microtest for FMOD's actually reached cookie/lock startup APIs.
 * Counter values/PIDs are per-host observations, never cross-machine constants. */
typedef unsigned int U;typedef unsigned long long Q;typedef void *H;
struct Counter {U low,high;};struct Guarded {struct Counter value;U guard;};
_Static_assert(sizeof(H)==4&&sizeof(struct Counter)==8,"Win32 startup clock widths");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetCurrentProcessId(void);
__declspec(dllimport) U __stdcall GetCurrentThreadId(void);
__declspec(dllimport) U __stdcall GetTickCount(void);
__declspec(dllimport) int __stdcall QueryPerformanceCounter(struct Counter *);
__declspec(dllimport) int __stdcall QueryPerformanceFrequency(struct Counter *);
__declspec(dllimport) U __stdcall InterlockedCompareExchange(volatile U *,U,U);
__declspec(dllimport) U __stdcall InterlockedExchange(volatile U *,U);
static U words[64],used;
static void put(U value){if(used>=64)__builtin_trap();words[used++]=value;}
static void failed(U stage){
    put(0x4641494c);put(stage);U done=0;
    if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();
    __builtin_trap();
}
U entry(void){
    put(0x434c3332);put(sizeof(H));SetLastError(0xface1234);
    put(GetCurrentProcessId());put(GetLastError());put(GetCurrentProcessId());
    SetLastError(0xface1234);put(GetCurrentThreadId());put(GetLastError());put(GetCurrentThreadId());
    struct Guarded frequency={{0,0},0x715e7a11},first={{0,0},0x715e7a11},last={{0,0},0x715e7a11};
    SetLastError(0xface1234);put((U)QueryPerformanceFrequency(&frequency.value));put(GetLastError());put(frequency.value.low);put(frequency.value.high);put(frequency.guard);
    SetLastError(0xface1234);U start=GetTickCount();put(start);put(GetLastError());
    SetLastError(0xface1234);put((U)QueryPerformanceCounter(&first.value));put(GetLastError());put(first.value.low);put(first.value.high);put(first.guard);
    /* The native fast user-page tick reader can exhaust two million calls
     * before a 32ms timer interval. Retain a finite iteration budget and the
     * independent parent's 20s deadline; report the exact failed stage. */
    U end=0,i=0;for(;i<100000000;i++){end=GetTickCount();if((U)(end-start)>=32)break;}if(i==100000000)failed(1);put(end);
    SetLastError(0xface1234);put((U)QueryPerformanceCounter(&last.value));put(GetLastError());put(last.value.low);put(last.value.high);put(last.guard);
    if(!frequency.value.low||frequency.value.high||(((Q)last.value.high<<32)|last.value.low)<(((Q)first.value.high<<32)|first.value.low))failed(2);
    struct {U before;volatile U value;U after;} atomic={0x715e7a11,0,0x715e7a11};
    for(U n=0;n<3;n++){
        U exchange=n==0?0xffffffff:n==1?7:0x80000000,comparand=n==2?0xffffffff:0;
        SetLastError(0xface1234);put(InterlockedCompareExchange(&atomic.value,exchange,comparand));put(GetLastError());put(atomic.value);put(atomic.before);put(atomic.after);
    }
    SetLastError(0xface1234);put(InterlockedExchange(&atomic.value,0xffffffff));put(GetLastError());put(atomic.value);put(atomic.before);put(atomic.after);
    put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
