/* Original MIT same-PE witness for the default mouse messages reached by DAO.
 * Hidden owned top-level windows; no synthesized system input or game code. */
typedef unsigned int U; typedef int L; typedef unsigned short W; typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetModuleHandleW(const W *); API H __stdcall GetStdHandle(U);
API int __stdcall WriteFile(H,const void *,U,U *,H);
API W __stdcall RegisterClassExW(const void *);
API H __stdcall CreateWindowExW(U,const W *,const W *,U,L,L,L,L,H,H,H,void *);
API L __stdcall DefWindowProcW(H,U,U,L); API int __stdcall DestroyWindow(H);
API H __stdcall GetCapture(void); API int __stdcall GetWindowInfo(H,U *);
static U words[96],used,contexts,seen_w,seen_l,delegate;
static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static L __stdcall procedure(H h,U msg,U w,L l){
 if(msg==0x7b){++contexts;seen_w=w;seen_l=(U)l;if(!delegate)return 0x13579bdf;}
 return DefWindowProcW(h,msg,w,l);
}
static U point(L x,L y){return ((U)x&0xffff)|(((U)y&0xffff)<<16);}
U entry(void){
 U before=stack();put(0x4d444633);put(1);
 H instance=GetModuleHandleW(0);
 U wc[]={48,8,(U)procedure,0,0,(U)instance,0,0,0,0,(U)L"AlkyholMouseDefaults",0};
 put(RegisterClassExW(wc)!=0);
 H h=CreateWindowExW(0,L"AlkyholMouseDefaults",L"Alkyhol hidden mouse defaults",0x80000000,120,160,320,240,0,0,instance,0);
 put(h!=0);U info[15]={60};put(GetWindowInfo(h,info)!=0);
 static const U messages[]={0x200,0x201,0x202,0x203,0x204,0x206,0x207,0x208,0x209};
 for(U i=0;i<sizeof(messages)/sizeof(messages[0]);++i){
  contexts=0;put(messages[i]);put((U)DefWindowProcW(h,messages[i],3,(L)point(3,4)));put(contexts);put(GetCapture()==0);
 }
 static const L coords[][2]={{3,4},{-7,-13},{32767,-32768},{3,4}};
 for(U i=0;i<4;++i){
  contexts=0;seen_w=seen_l=0;delegate=i==3;
  U packed=point(coords[i][0],coords[i][1]);
  U expected=point((L)info[5]+coords[i][0],(L)info[6]+coords[i][1]);
  put(packed);put((U)DefWindowProcW(h,0x205,0x1234,(L)packed));put(contexts);put(seen_w==(U)h);put(seen_l==expected);put(GetCapture()==0);
 }
 put((U)DefWindowProcW(h,0x7b,(U)h,(L)point((L)info[5]+3,(L)info[6]+4)));put(contexts==1);
 put(DestroyWindow(h)!=0);put(stack()==before);put(used+1);
 U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
