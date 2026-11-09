/* Original MIT save-slot prefix enumeration witness. No game names or files. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
#define API __declspec(dllimport)
struct DataW{U attr,times[6],high,low,reserved[2];W name[260],alternate[14];};
struct DataA{U attr,times[6],high,low,reserved[2];char name[260],alternate[14];};
_Static_assert(sizeof(struct DataW)==592&&sizeof(struct DataA)==320,"find layouts");
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API U __stdcall GetModuleFileNameW(H,W *,U);API U __stdcall GetLastError(void);
API H __stdcall FindFirstFileW(const W *,struct DataW *);API int __stdcall FindNextFileW(H,struct DataW *);
API H __stdcall FindFirstFileA(const char *,struct DataA *);API int __stdcall FindNextFileA(H,struct DataA *);
API int __stdcall FindClose(H);
static U words[24],used;static W path[1024];static char ansi[1024];static U at;
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static void put(U n){if(used>=24)__builtin_trap();words[used++]=n;}
static void mask(const W *suffix){U i=0;do{path[at+i]=suffix[i];}while(suffix[i++]);for(i=0;path[i];i++){if(path[i]>127)__builtin_trap();ansi[i]=(char)path[i];}ansi[i]=0;}
static U equal(const W *w,const char *a){U i=0;while(w[i]&&(U)w[i]==(unsigned char)a[i])i++;return w[i]==0&&a[i]==0;}
U entry(void){
 U before=stack();put(0x46505246);put(1);U n=GetModuleFileNameW(0,path,1024);if(!n||n>=1023)__builtin_trap();at=n;while(at&&path[at-1]!='\\')at--;if(!at)__builtin_trap();
 for(U kind=0;kind<2;kind++){
  struct DataW w;struct DataA a;mask(L"Slot_*");H h=kind?FindFirstFileA(ansi,&a):FindFirstFileW(path,&w);put(h!=0&&h!=(H)-1);if(h==0||h==(H)-1)__builtin_trap();
  U count=0,seen=0,valid=1;
  do{
   if(kind){for(U i=0;i<260;i++)w.name[i]=(unsigned char)a.name[i];w.attr=a.attr;w.high=a.high;w.low=a.low;}
   U bit=equal(w.name,"Slot_")?1:equal(w.name,"Slot_001")?2:equal(w.name,"slot_002")?4:equal(w.name,"Slot_data.dat")?8:0;
   if(!bit||(seen&bit)||w.high||((w.attr&16)!=0)!=(bit!=8)||(bit==8&&w.low!=3))valid=0;seen|=bit;count++;
  }while(kind?FindNextFileA(h,&a):FindNextFileW(h,&w));
  put(count==4&&seen==15&&valid);put(GetLastError()==18);put(FindClose(h)!=0);
  mask(L"Absent_*");h=kind?FindFirstFileA(ansi,&a):FindFirstFileW(path,&w);put(h==(H)-1&&GetLastError()==2);
  mask(L"absent\\Slot_*");h=kind?FindFirstFileA(ansi,&a):FindFirstFileW(path,&w);put(h==(H)-1&&GetLastError()==3);
 }
 put(before==stack());put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
