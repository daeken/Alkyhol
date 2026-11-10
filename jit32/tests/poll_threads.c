/* MIT. DAO-reached zero-timeout polling plus real worker progress. */
typedef unsigned int U;
typedef void *H;
#define API __declspec(dllimport)
API H __stdcall CreateEventW(void *,int,int,const unsigned short *);
API int __stdcall SetEvent(H);
API H __stdcall CreateThread(void *,U,U(__stdcall *)(void *),void *,U,U *);
API U __stdcall WaitForSingleObject(H,U);
API int __stdcall GetExitCodeThread(H,U *);
API int __stdcall InterlockedExchangeAdd(volatile int *,int);
API int __stdcall CloseHandle(H);
API H __stdcall GetStdHandle(U);
API int __stdcall WriteFile(H,const void *,U,U *,void *);
static H ready,stop;
static volatile int polls;
static U words[12],used;
static void put(U value){words[used++]=value;}
static U __stdcall worker(void *unused){
    (void)unused;
    while(WaitForSingleObject(stop,0)==258){
        if(!polls)SetEvent(ready);
        InterlockedExchangeAdd(&polls,1);
    }
    return 0x71717171;
}
U entry(void){
    H automatic=CreateEventW(0,0,0,0);
    if(!automatic)__builtin_trap();
    put(0x504f4c4c);put(4);
    put(WaitForSingleObject(automatic,0)==258);
    if(!SetEvent(automatic))__builtin_trap();
    put(WaitForSingleObject(automatic,0)==0);
    put(WaitForSingleObject(automatic,0)==258);
    ready=CreateEventW(0,1,0,0);stop=CreateEventW(0,1,0,0);
    H thread=CreateThread(0,0,worker,0,0,0);
    if(!ready||!stop||!thread)__builtin_trap();
    put(WaitForSingleObject(ready,(U)-1)==0);
    put(WaitForSingleObject(thread,0)==258);
    if(!SetEvent(stop))__builtin_trap();
    put(WaitForSingleObject(thread,(U)-1)==0);
    U status=0;put(GetExitCodeThread(thread,&status)&&status==0x71717171);
    put(polls>0);
    put(CloseHandle(thread)&&CloseHandle(ready)&&CloseHandle(stop)&&CloseHandle(automatic));
    put(12);U done=0;
    if(!WriteFile(GetStdHandle((U)-11),words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();
    return 0;
}
