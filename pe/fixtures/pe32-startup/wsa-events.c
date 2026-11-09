/* Original MIT: manual event-set WSA waits and manual-reset aliases reached by DAO. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) H __stdcall CreateEventA(H,int,int,const char*);
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
static U data[100],out[80],used;static void put(U n){out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U __stdcall worker(H event){Sleep(2);return WSASetEvent(event)?5:9;}
U entry(void){U before=stack();put(0x45534157);put(3);put(WSAStartup(0x202,data));H a=WSACreateEvent(),b=WSACreateEvent();put(a&&b);if(!a||!b)ExitProcess(2);
 put(WSAWaitForMultipleEvents(1,&a,0,0,0));put(WSASetEvent(a));put(WSAWaitForMultipleEvents(1,&a,0,0,0));put(WSAWaitForMultipleEvents(1,&a,1,0,0));put(WSAResetEvent(a));
 U id=0;H thread=CreateThread(0,0,worker,a,0,&id);put(thread!=0&&id!=0);if(!thread)ExitProcess(3);
 put(WSAWaitForMultipleEvents(1,&a,0,5000,0));U code=0;put(WaitForSingleObject(thread,5000)==0&&GetExitCodeThread(thread,&code)&&code==5);
 put(WSAWaitForMultipleEvents(0,&a,0,0,0));put(WSAGetLastError());put(WSAWaitForMultipleEvents(1,&b,0,2,0));
 H both[2]={a,b},duplicate[2]={a,a};put(WSAResetEvent(a));put(WSAWaitForMultipleEvents(2,both,0,0,0));put(WSASetEvent(b));put(WSAWaitForMultipleEvents(2,both,0,0,0));
 put(WSASetEvent(a));put(WSAWaitForMultipleEvents(2,both,0,0,0));put(WSAWaitForMultipleEvents(2,both,1,0,0));put(WSAResetEvent(a));put(WSAWaitForMultipleEvents(2,both,1,2,0));
 put(WSASetEvent(a));put(WSAWaitForMultipleEvents(2,both,1,0,0));put(WSAWaitForMultipleEvents(2,duplicate,0,0,0));put(WSAWaitForMultipleEvents(2,duplicate,1,0,0));put(WSAGetLastError());
 put(WSAResetEvent(a)&&WSAResetEvent(b));U second_id=0;H second=CreateThread(0,0,worker,a,0,&second_id);put(second!=0&&second_id!=0);if(!second)ExitProcess(4);
 put(WSAWaitForMultipleEvents(2,both,0,5000,0));code=0;put(WaitForSingleObject(second,5000)==0&&GetExitCodeThread(second,&code)&&code==5&&CloseHandle(second));
 H x=CreateEventA(0,0,0,0),y=CreateEventA(0,0,1,0);put(x&&y);if(!x||!y)ExitProcess(5);H automatic[2]={x,y},mixed[2]={a,x};
 put(WSAWaitForMultipleEvents(2,automatic,0,0,0));put(WSAWaitForMultipleEvents(2,automatic,0,0,0));put(WSASetEvent(x));put(WSAWaitForMultipleEvents(2,automatic,1,0,0));
 put(WaitForSingleObject(x,0));put(WaitForSingleObject(x,0));put(WSASetEvent(x)&&WSASetEvent(y));put(WSAWaitForMultipleEvents(2,automatic,1,0,0));put(WaitForSingleObject(x,0)==258&&WaitForSingleObject(y,0)==258);
 put(WSASetEvent(x));put(WSAWaitForMultipleEvents(2,mixed,1,0,0));put(WaitForSingleObject(a,0));put(WaitForSingleObject(x,0));put(WSASetEvent(x)&&WSASetEvent(x));put(WaitForSingleObject(x,0));put(WaitForSingleObject(x,0));
 H third=CreateThread(0,0,worker,x,0,&id);put(third!=0);if(!third)ExitProcess(6);put(WSAWaitForMultipleEvents(2,automatic,0,5000,0));code=0;put(WaitForSingleObject(third,5000)==0&&GetExitCodeThread(third,&code)&&code==5&&CloseHandle(third));put(WSACloseEvent(x));put(WSACloseEvent(y));
 put(WSACloseEvent(a));put(WSACloseEvent(b));put(WSACloseEvent(a));put(WSAGetLastError());put(WSACleanup());put(CloseHandle(thread));put(stack()==before);put(used+1);
 U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
