/* Original MIT witness for the capture calls reached by original DAO. */
typedef unsigned int U; typedef int L; typedef unsigned short W; typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetModuleHandleW(const W *); API H __stdcall GetStdHandle(U);
API int __stdcall WriteFile(H,const void *,U,U *,H);
API W __stdcall RegisterClassExW(const void *);
API H __stdcall CreateWindowExW(U,const W *,const W *,U,L,L,L,L,H,H,H,void *);
API L __stdcall DefWindowProcW(H,U,U,L); API int __stdcall ShowWindow(H,int);
API int __stdcall DestroyWindow(H); API H __stdcall SetCapture(H);
API int __stdcall ReleaseCapture(void); API H __stdcall GetCapture(void);
API void __stdcall Sleep(U); API int __stdcall PeekMessageW(void *,H,U,U,U);
API L __stdcall DispatchMessageW(const void *);
static U words[32],used,changed,bad; static H expected;
static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static L __stdcall procedure(H h,U msg,U w,L l){
 if(msg==0x215){++changed;if(w||l!=(L)expected||GetCapture()!=expected)bad=1;}
 return DefWindowProcW(h,msg,w,l);
}
U entry(void){
 U before=stack();put(0x43415032);put(1);H instance=GetModuleHandleW(0);
 U wc[]={48,0,(U)procedure,0,0,(U)instance,0,0,0,0,(U)L"AlkyholCaptureWitness",0};
 put(RegisterClassExW(wc)!=0);
 H h=CreateWindowExW(0,L"AlkyholCaptureWitness",L"Alkyhol PE32 capture witness",0xcf0000,100,100,320,240,0,0,instance,0);
 put(h!=0);ShowWindow(h,1);
 /* Allow real asynchronous window activation to settle on either host. */
 for(U i=0;i<20;++i){U msg[7];if(PeekMessageW(msg,0,0,0,1))DispatchMessageW(msg);Sleep(10);}
 put(GetCapture()==0);
 put(SetCapture(h)==0);put(GetCapture()==h);put(SetCapture(h)==h);put(changed==0);
 expected=0;put(ReleaseCapture()!=0);put(GetCapture()==0);put(changed==1&&!bad);
 put(SetCapture(h)==0);put(GetCapture()==h);put(ReleaseCapture()!=0);put(changed==2&&!bad);
 put(DestroyWindow(h)!=0);put(stack()==before);put(used+1);
 U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
