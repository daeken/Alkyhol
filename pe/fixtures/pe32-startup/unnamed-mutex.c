/* Original MIT PE32 mutex/thread witness; no game data or canned worker results. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) H __stdcall CreateMutexA(H,int,const char*);
__declspec(dllimport) H __stdcall CreateMutexW(H,int,const unsigned short*);
__declspec(dllimport) int __stdcall ReleaseMutex(H);
__declspec(dllimport) U __stdcall WaitForSingleObject(H,U);
__declspec(dllimport) H __stdcall CreateEventW(H,int,int,const unsigned short*);
__declspec(dllimport) int __stdcall SetEvent(H);
__declspec(dllimport) H __stdcall CreateThread(H,U,U(__stdcall*)(H),H,U,U*);
__declspec(dllimport) int __stdcall GetExitCodeThread(H,U*);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static H mutex,ready,acquired;static volatile U worker_words[7];static U words[80],used;
static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U __stdcall worker(H parameter){(void)parameter;
 worker_words[0]=WaitForSingleObject(mutex,0);SetLastError(0xface1234);worker_words[1]=ReleaseMutex(mutex);worker_words[2]=GetLastError();
 worker_words[3]=WaitForSingleObject(mutex,2);worker_words[4]=SetEvent(ready);
 worker_words[5]=WaitForSingleObject(mutex,5000);worker_words[6]=SetEvent(acquired);
 return 0x3344; /* Actual thread exit must abandon the mutex it still owns. */
}
U entry(void){U before=stack();put(0x584d5541);put(1);
 SetLastError(0xface1234);mutex=CreateMutexA(0,0,0);put(mutex!=0);put(GetLastError());if(!mutex)ExitProcess(2);
 put(WaitForSingleObject(mutex,0));put(WaitForSingleObject(mutex,0));ready=CreateEventW(0,1,0,0);acquired=CreateEventW(0,1,0,0);put(ready&&acquired);
 U thread_id=0;H thread=CreateThread(0,0,worker,0,0,&thread_id);put(thread!=0&&thread_id!=0);if(!thread)ExitProcess(3);
 put(WaitForSingleObject(ready,5000));put(ReleaseMutex(mutex));put(WaitForSingleObject(acquired,2));put(ReleaseMutex(mutex));
 put(WaitForSingleObject(thread,5000));U exit=0;put(GetExitCodeThread(thread,&exit)&&exit==0x3344);
 for(U i=0;i<7;i++)put(worker_words[i]);
 put(WaitForSingleObject(mutex,0));put(ReleaseMutex(mutex));SetLastError(0xface1234);put(ReleaseMutex(mutex));put(GetLastError());
 put(CloseHandle(mutex));put(CloseHandle(thread));put(CloseHandle(ready));put(CloseHandle(acquired));
 H wide=CreateMutexW(0,1,0);put(wide!=0);put(WaitForSingleObject(wide,0));put(ReleaseMutex(wide));put(ReleaseMutex(wide));put(ReleaseMutex(wide));put(GetLastError());put(CloseHandle(wide));
 H owned=CreateMutexA(0,1,0);put(owned!=0&&CloseHandle(owned));put(stack()==before);put(used+1);
 U done=0;int ok=WriteFile(GetStdHandle((U)-11),words,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
