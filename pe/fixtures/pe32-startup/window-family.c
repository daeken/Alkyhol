/* Original MIT test: original x86 Win32 callbacks and an owned native window. */
typedef unsigned int U;typedef int L;typedef unsigned short W;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetModuleHandleW(const W *);API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API W __stdcall RegisterClassExW(const void *);API H __stdcall CreateWindowExW(U,const W *,const W *,U,L,L,L,L,H,H,H,void *);
API L __stdcall DefWindowProcW(H,U,U,L);API int __stdcall AdjustWindowRect(void *,U,int);API int __stdcall ShowWindow(H,int);API int __stdcall UpdateWindow(H);
API int __stdcall GetClientRect(H,void *);API int __stdcall GetWindowRect(H,void *);API L __stdcall GetWindowLongW(H,L);API L __stdcall SetWindowLongW(H,L,L);
API int __stdcall SetWindowTextW(H,const W *);API int __stdcall IsWindowVisible(H);API int __stdcall DestroyWindow(H);API int __stdcall UnregisterClassA(const char *,H);
API U __stdcall GetLastError(void);API int __stdcall GetSystemMetrics(int);API U __stdcall GetClassLongA(H,int);API int __stdcall ClientToScreen(H,void *);API int __stdcall ScreenToClient(H,void *);
static U words[96],used,tag=0x715e7a11,seen_nc,seen_create,seen_destroy,creation_style,valid=1;static H instance;static L outer_w,outer_h;
static void put(U value){words[used++]=value;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static L __stdcall procedure(H h,U msg,U w,L l){
 if(msg==0x81){U *p=(U *)l;++seen_nc;creation_style=p[8];if(p[0]!=(U)&tag||p[1]!=(U)instance||p[2]||p[3]||p[4]!=(U)outer_h||p[5]!=(U)outer_w||p[6]!=100||p[7]!=100)valid=0;if(SetWindowLongW(h,-21,(L)tag)!=0)valid=0;}
 if(msg==1){++seen_create;if((U)GetWindowLongW(h,-21)!=tag)valid=0;}
 if(msg==2)++seen_destroy;
 return DefWindowProcW(h,msg,w,l);
}
U entry(void){put(0x57494e32);put(4);U before=stack();instance=GetModuleHandleW(0);U wc[]={48,3,(U)procedure,0,4,(U)instance,0,0,0,0,(U)L"AlkyholOwnedWindowFamily",0};
 put(RegisterClassExW(wc)!=0);put(GetSystemMetrics(0)>0&&GetSystemMetrics(1)>0);L rect[]={0,0,240,120};put(AdjustWindowRect(rect,0xcf0000,0)!=0);outer_w=rect[2]-rect[0];outer_h=rect[3]-rect[1];
 H hwnd=CreateWindowExW(0,L"AlkyholOwnedWindowFamily",L"Alkyhol owned PE32 window",0xcf0000,100,100,outer_w,outer_h,0,0,instance,&tag);put(hwnd!=0);put(seen_nc);put(seen_create);put(valid);put((U)GetWindowLongW(hwnd,-21)==tag);put(SetWindowLongW(hwnd,0,1234)==0);put(GetWindowLongW(hwnd,0)==1234);put(GetClassLongA(hwnd,-26)==3);
 L client[4];put(GetClientRect(hwnd,client)!=0);put(client[0]==0&&client[1]==0&&client[2]==240&&client[3]==120);put(GetWindowRect(hwnd,rect)!=0);put(rect[0]==100&&rect[1]==100&&rect[2]-rect[0]==outer_w&&rect[3]-rect[1]==outer_h);
 L point[]={5,7};put(ClientToScreen(hwnd,point)!=0);put(ScreenToClient(hwnd,point)!=0);put(point[0]==5&&point[1]==7);put(SetWindowTextW(hwnd,L"Alkyhol owned PE32 window: shown")!=0);put(ShowWindow(hwnd,1)==0);put(IsWindowVisible(hwnd)!=0);put(UpdateWindow(hwnd)!=0);put(UnregisterClassA("AlkyholOwnedWindowFamily",instance)==0);put(GetLastError());
 put(DestroyWindow(hwnd)!=0);put(seen_destroy);put(UnregisterClassA("AlkyholOwnedWindowFamily",instance)!=0);put(stack()==before);put(creation_style);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;}
