/* Original MIT: single-event WSA waits and manual-reset aliases reached by DAO. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) U __stdcall WSAStartup(U,void*);
__declspec(dllimport) U __stdcall WSACleanup(void);
__declspec(dllimport) U __stdcall WSAGetLastError(void);
__declspec(dllimport) H __stdcall WSACreateEvent(void);
__declspec(dllimport) int __stdcall WSASetEvent(H);
__declspec(dllimport) int __stdcall WSAResetEvent(H);
__declspec(dllimport) int __stdcall WSACloseEvent(H);
__declspec(dllimport) U __stdcall WSAWaitForMultipleEvents(U,const H*,int,U,int);
__declspec(dllimport) H __stdcall CreateThread(H,U,U(__stdcall*)(H),H,U,U*);
__declspec(dllimport) U __stdcall WaitForSingleObject(H,U);
__declspec(dllimport) int __stdcall GetExitCodeThread(H,U*);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) void __stdcall Sleep(U);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U data[100],out[40],used;static void put(U n){out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U __stdcall worker(H event){Sleep(2);return WSASetEvent(event)?5:9;}
U entry(void){U before=stack();put(0x45534157);put(1);put(WSAStartup(0x202,data));H a=WSACreateEvent(),b=WSACreateEvent();put(a&&b);if(!a||!b)ExitProcess(2);
 put(WSAWaitForMultipleEvents(1,&a,0,0,0));put(WSASetEvent(a));put(WSAWaitForMultipleEvents(1,&a,0,0,0));put(WSAWaitForMultipleEvents(1,&a,1,0,0));put(WSAResetEvent(a));
 U id=0;H thread=CreateThread(0,0,worker,a,0,&id);put(thread!=0&&id!=0);if(!thread)ExitProcess(3);
 put(WSAWaitForMultipleEvents(1,&a,0,5000,0));U code=0;put(WaitForSingleObject(thread,5000)==0&&GetExitCodeThread(thread,&code)&&code==5);
 put(WSAWaitForMultipleEvents(0,&a,0,0,0));put(WSAGetLastError());put(WSAWaitForMultipleEvents(1,&b,0,2,0));
 put(WSACloseEvent(a));put(WSACloseEvent(b));put(WSACloseEvent(a));put(WSAGetLastError());put(WSACleanup());put(CloseHandle(thread));put(stack()==before);put(used+1);
 U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
