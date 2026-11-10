/* Original MIT witness: query actual unconfined desktop bounds. No cursor
 * motion, rectangle confinement, synthetic mouse input, or game code. */
typedef unsigned int U; typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API int __stdcall GetClipCursor(void *);API int __stdcall ClipCursor(const void *);
API int __stdcall GetSystemMetrics(int);API int __stdcall ShowCursor(int);
static U words[160],used;static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){
 U before=stack();put(0x434c5032);put(1);put(ClipCursor(0)!=0);
 for(U n=0;n<24;++n){
  struct {U before;U rect[4];U after;} box={0x12345678,{0,0,0,0},0x87654321};
  U expected[4];expected[0]=(U)GetSystemMetrics(76);expected[1]=(U)GetSystemMetrics(77);
  expected[2]=expected[0]+(U)GetSystemMetrics(78);expected[3]=expected[1]+(U)GetSystemMetrics(79);
  put(GetClipCursor(box.rect)!=0);put(box.before==0x12345678&&box.after==0x87654321);
  put(box.rect[0]==expected[0]&&box.rect[1]==expected[1]&&box.rect[2]==expected[2]&&box.rect[3]==expected[3]);
  put(stack()==before);
 }
 const int args[8]={0,0,1,7,1,0,0,1};
 const int counts[8]={-1,-2,-1,0,1,0,-1,0};
 for(U n=0;n<8;++n){put(ShowCursor(args[n])==counts[n]);put(stack()==before);}
 put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
