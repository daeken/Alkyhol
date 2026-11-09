/* Original MIT: real socket readiness and Win32 counted fd_sets reached by DAO.
 * All test packets stay on an actual owned IPv4 loopback socket. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) U __stdcall WSAStartup(U,void*);
__declspec(dllimport) U __stdcall WSACleanup(void);
__declspec(dllimport) U __stdcall WSAGetLastError(void);
__declspec(dllimport) U __stdcall socket(U,U,U);
__declspec(dllimport) U __stdcall closesocket(U);
__declspec(dllimport) U __stdcall bind(U,const void*,U);
__declspec(dllimport) U __stdcall getsockname(U,void*,U*);
__declspec(dllimport) U __stdcall sendto(U,const void*,U,U,const void*,U);
__declspec(dllimport) U __stdcall recvfrom(U,void*,U,U,void*,U*);
__declspec(dllimport) U __stdcall select(U,void*,void*,void*,const void*);
__declspec(dllimport) H __stdcall GetModuleHandleA(const char*);
__declspec(dllimport) H __stdcall GetProcAddress(H,const char*);
__declspec(dllimport) H __stdcall CreateThread(H,U,U(__stdcall*)(H),H,U,U*);
__declspec(dllimport) U __stdcall WaitForSingleObject(H,U);
__declspec(dllimport) int __stdcall GetExitCodeThread(H,U*);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) void __stdcall Sleep(U);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U data[100],out[64],used,s,endpoint[4];
static void put(U n){if(used>=64)__builtin_trap();out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
#define GUARD 0x715e7a11
struct Set {U before,count,items[2],after;};
static void init(struct Set* set){set->before=GUARD;set->count=1;set->items[0]=s;set->items[1]=GUARD;set->after=GUARD;}
static U guards(const struct Set* set){return set->before==GUARD&&set->items[0]==s&&set->items[1]==GUARD&&set->after==GUARD;}
static U __stdcall sender(H ignored){(void)ignored;Sleep(2);return sendto(s,"ping",4,0,endpoint,16)==4?5:9;}
/* Literal RET leaves its argument for the startup caller, unlike sender's RET 4. */
__declspec(naked) static U __stdcall caller_cleaned(H parameter __attribute__((unused))){__asm__ volatile("movl 4(%esp),%eax; retl");}
U entry(void){U before=stack();put(0x4c455353);put(2);put(WSAStartup(0x202,data));s=socket(2,2,17);put(s!=(U)-1);if(s==(U)-1)ExitProcess(2);
 U address[4]={2,0x0100007f,0,0};put(bind(s,address,16));U extent=16;put(getsockname(s,endpoint,&extent));put(GetProcAddress(GetModuleHandleA("ws2_32.dll"),(const char*)18)==(H)select);
 struct Set read,write,except;init(&read);init(&write);init(&except);U time[4]={GUARD,0,0,GUARD};
 put(select(999,&read.count,0,0,time+1));put(read.count==0&&guards(&read));
 init(&read);put(select(0,&read.count,&write.count,&except.count,time+1));put(read.count==0&&write.count==1&&except.count==0&&guards(&read)&&guards(&write)&&guards(&except));put(time[0]==GUARD&&time[1]==0&&time[2]==0&&time[3]==GUARD);
 init(&read);time[2]=2000;put(select(0,&read.count,0,0,time+1));put(read.count==0&&guards(&read)&&time[1]==0&&time[2]==2000&&time[0]==GUARD&&time[3]==GUARD);
 put(select(0,0,0,0,time+1));put(WSAGetLastError());
 init(&read);read.items[0]=(U)-1;put(select(0,&read.count,0,0,time+1));put(WSAGetLastError());put(read.count==1&&read.items[0]==(U)-1&&read.items[1]==GUARD&&read.before==GUARD&&read.after==GUARD);
 init(&read);time[1]=(U)-1;put(select(0,&read.count,0,0,time+1));put(WSAGetLastError());put(read.count==1&&guards(&read)&&time[1]==(U)-1&&time[2]==2000);
 time[1]=5;time[2]=0;put(sendto(s,"hello",5,0,endpoint,16));put(select(0,&read.count,0,0,time+1));put(read.count==1&&guards(&read)&&time[1]==5&&time[2]==0);
 init(&write);init(&except);time[1]=0;put(select(0,&read.count,&write.count,&except.count,time+1));put(read.count==1&&write.count==1&&except.count==0&&guards(&read)&&guards(&write)&&guards(&except));
 char buffer[8];for(U i=0;i<8;i++)buffer[i]=0x5a;put(recvfrom(s,buffer,8,0,0,0));put(buffer[0]=='h'&&buffer[4]=='o'&&buffer[5]==0x5a&&buffer[7]==0x5a);
 U id=0;H thread=CreateThread(0,0,sender,0,0,&id);put(thread&&id);if(!thread)ExitProcess(3);init(&read);put(select(0,&read.count,0,0,0));U code=0;put(WaitForSingleObject(thread,5000)==0&&GetExitCodeThread(thread,&code)&&code==5&&CloseHandle(thread));
 put(recvfrom(s,buffer,8,0,0,0));put(buffer[0]=='p'&&buffer[1]=='i'&&buffer[2]=='n'&&buffer[3]=='g'&&buffer[5]==0x5a);
 H plain=CreateThread(0,0,caller_cleaned,(H)0x1234,0,&id);put(plain&&id);if(!plain)ExitProcess(4);put(WaitForSingleObject(plain,5000));code=0;put(GetExitCodeThread(plain,&code)&&code==0x1234);put(CloseHandle(plain));
 put(closesocket(s));put(WSACleanup());put(stack()==before);put(used+1);U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
