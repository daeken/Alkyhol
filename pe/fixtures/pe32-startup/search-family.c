/* Original MIT wildcard A/W and mixed-encoding enumeration probe. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetModuleFileNameA(H,char *,U);__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall FindFirstFileA(const char *,void *);__declspec(dllimport) H __stdcall FindFirstFileW(const W *,void *);
__declspec(dllimport) int __stdcall FindNextFileA(H,void *);__declspec(dllimport) int __stdcall FindNextFileW(H,void *);__declspec(dllimport) int __stdcall FindClose(H);
static char path[512];static W wide[512];static U prefix,words[96],used;
static void put(U n){words[used++]=n;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U equal(const char *a,const char *b){U i=0;do{if(a[i]!=b[i])return 0;}while(a[i++]);return 1;}
U entry(void){U n=GetModuleFileNameA(0,path,512);if(!n||n>=448)__builtin_trap();while(n&&path[n-1]!='\\')--n;prefix=n;const char root[]="root\\";for(U i=0;i<sizeof(root)-1;++i)path[prefix++]=root[i];U before=stack();put(0x53454332);put(4);
 for(U test=0;test<6;++test){const char *mask=test<3?"*":"*.txt";U at=0;do{path[prefix+at]=mask[at];}while(mask[at++]);for(U i=0;path[i];++i)wide[i]=(W)(unsigned char)path[i];wide[prefix+at-1]=0;
  struct {U before;U data[148];U after;} w;struct {U before;U data[80];U after;} a;w.before=w.after=a.before=a.after=0x715e7a11;U mode=test%3;H h=mode==1?FindFirstFileA(path,a.data):FindFirstFileW(wide,w.data);if(h==(H)-1)__builtin_trap();U seen=0,count=0,valid=1,duplicate=0,encoding=mode==1;
  for(;;){U *data=encoding?a.data:w.data;char name[260];for(U i=0;i<260;++i){name[i]=encoding?((char *)data)[44+i]:(char)((W *)((char *)data+44))[i];if(!name[i])break;if(i==259)__builtin_trap();}
   const char *names[]={".","..","alpha.txt","Beta.TXT","plain","folder"};const U sizes[]={0,0,3,5,2,0};U index=6;for(U i=0;i<6;++i)if(equal(name,names[i]))index=i;if(index==6)__builtin_trap();U bit=1u<<index;if(seen&bit)duplicate=1;seen|=bit;++count;
   if(data[0]!=(index<2||index==5?16u:32u)||data[7]||data[8]!=sizes[index])valid=0;
   encoding=mode!=0;if(!(encoding?FindNextFileA(h,a.data):FindNextFileW(h,w.data)))break;
  }
  put(seen);put(count);put(valid);put(!duplicate);put(GetLastError());put(FindClose(h)!=0);put(w.before==0x715e7a11&&w.after==0x715e7a11&&a.before==0x715e7a11&&a.after==0x715e7a11);
 }
 put(stack()==before);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
