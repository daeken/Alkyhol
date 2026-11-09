/* Original MIT: SO_BROADCAST set/get on actual UDP sockets, reached by DAO.
 * No packet is sent by this witness. */
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
__declspec(dllimport) U __stdcall WSARecvFrom(U,const void*,U,U*,U*,void*,U*,void*,H);
__declspec(dllimport) U __stdcall ioctlsocket(U,U,U*);
__declspec(dllimport) void __stdcall Sleep(U);
__declspec(dllimport) U __stdcall setsockopt(U,U,U,const void*,U);
__declspec(dllimport) U __stdcall getsockopt(U,U,U,void*,U*);
__declspec(dllimport) H __stdcall GetModuleHandleA(const char*);
__declspec(dllimport) H __stdcall GetProcAddress(H,const char*);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U data[100],out[80],used;static void put(U n){if(used>=80)__builtin_trap();out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U before=stack();put(0x42534f53);put(1);put(WSAStartup(0x202,data));U s=socket(2,2,17);put(s!=(U)-1);if(s==(U)-1)ExitProcess(2);H module=GetModuleHandleA("ws2_32.dll");put(module&&GetProcAddress(module,(const char*)21)==(H)setsockopt&&GetProcAddress(module,(const char*)7)==(H)getsockopt&&GetProcAddress(module,(const char*)2)==(H)bind&&GetProcAddress(module,(const char*)6)==(H)getsockname&&GetProcAddress(module,(const char*)20)==(H)sendto&&GetProcAddress(module,(const char*)17)==(H)recvfrom);
#ifdef ASYNC_EXCLUSION
 char async_buffer[8]={0};U async_descriptor[2]={8,(U)async_buffer},async_bytes=0x715e7a11,async_flags=0,overlapped[5]={0};
 put(WSARecvFrom(s,async_descriptor,1,&async_bytes,&async_flags,0,0,overlapped,0));put(WSAGetLastError());
 put(async_bytes==0x715e7a11&&async_flags==0&&overlapped[0]==0&&overlapped[1]==0&&overlapped[2]==0&&overlapped[3]==0&&overlapped[4]==0);
 put(closesocket(s));put(WSACleanup());put(stack()==before);put(used+1);U async_done=0;int async_ok=WriteFile(GetStdHandle((U)-11),out,used*4,&async_done,0);ExitProcess(async_ok&&async_done==used*4?0:8);return 8;
#endif
 U address[4]={2,0x0100007f,0,0};put(bind(s,address,16));
 U endpoint[6]={0x715e7a11,0,0,0,0,0x715e7a11},extent[3]={0x715e7a11,16,0x715e7a11};
 put(getsockname(s,endpoint+1,extent+1));put((endpoint[1]&0xffff)==2&&(endpoint[1]>>16)!=0&&endpoint[2]==0x0100007f&&extent[1]==16);
 put(endpoint[0]==0x715e7a11&&endpoint[5]==0x715e7a11&&extent[0]==0x715e7a11&&extent[2]==0x715e7a11);
 for(U frame=0;frame<2;frame++){
  U enabled=frame?0:1;put(setsockopt(s,0xffff,0x20,&enabled,4));U values[3]={0x715e7a11,99,0x715e7a11},length[3]={0x715e7a11,4,0x715e7a11};
  put(getsockopt(s,0xffff,0x20,values+1,length+1));put(values[1]==enabled&&length[1]==4);put(values[0]==0x715e7a11&&values[2]==0x715e7a11&&length[0]==0x715e7a11&&length[2]==0x715e7a11);
 }
 U mode=1;put(ioctlsocket(s,0x8004667e,&mode));
 put(sendto(s,"hello",5,0,endpoint+1,16));
 struct {U before;char data[8];U after;} packet={0x715e7a11,{0},0x715e7a11};for(U i=0;i<8;i++)packet.data[i]=0x5a;
 U source[6]={0x715e7a11,0,0,0,0,0x715e7a11},source_length[3]={0x715e7a11,16,0x715e7a11};
 U received=(U)-1;
 for(U attempt=0;attempt<1000;attempt++){
  source_length[1]=16;received=recvfrom(s,packet.data,8,0,source+1,source_length+1);
  if(received!=(U)-1)break;if(WSAGetLastError()!=10035)break;Sleep(1);
 }
 put(received);put(packet.data[0]=='h'&&packet.data[1]=='e'&&packet.data[2]=='l'&&packet.data[3]=='l'&&packet.data[4]=='o'&&packet.data[5]==0x5a&&packet.data[7]==0x5a&&packet.before==0x715e7a11&&packet.after==0x715e7a11);
 put(source[1]==endpoint[1]&&source[2]==endpoint[2]&&source_length[1]==16&&source[0]==0x715e7a11&&source[5]==0x715e7a11&&source_length[0]==0x715e7a11&&source_length[2]==0x715e7a11);
 put(recvfrom(s,packet.data,8,0,0,0));put(WSAGetLastError());
 put(sendto(s,"world",5,0,endpoint+1,16));
 U descriptor[4]={0x715e7a11,8,(U)packet.data,0x715e7a11},received_bytes=0,flags=0,receive_status=(U)-1;
 for(U attempt=0;attempt<1000;attempt++){
  source_length[1]=16;receive_status=WSARecvFrom(s,descriptor+1,1,&received_bytes,&flags,source+1,source_length+1,0,0);
  if(receive_status!=(U)-1)break;if(WSAGetLastError()!=10035)break;Sleep(1);
 }
 put(receive_status);put(received_bytes);put(packet.data[0]=='w'&&packet.data[1]=='o'&&packet.data[2]=='r'&&packet.data[3]=='l'&&packet.data[4]=='d'&&packet.data[5]==0x5a);
 put(flags==0&&descriptor[0]==0x715e7a11&&descriptor[3]==0x715e7a11&&source[1]==endpoint[1]&&source[2]==endpoint[2]&&source_length[1]==16&&source[0]==0x715e7a11&&source[5]==0x715e7a11&&source_length[0]==0x715e7a11&&source_length[2]==0x715e7a11);
 put(closesocket(s));U enabled=1;put(setsockopt(s,0xffff,0x20,&enabled,4));put(WSAGetLastError());put(WSACleanup());put(stack()==before);put(used+1);
 U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
