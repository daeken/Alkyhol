/* Original MIT A/W full-path probe; expected paths derive from the owned cwd. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W *,U);__declspec(dllimport) int __stdcall SetCurrentDirectoryW(const W *);
__declspec(dllimport) U __stdcall GetFullPathNameW(const W *,U,W *,W **);__declspec(dllimport) U __stdcall GetFullPathNameA(const char *,U,char *,char **);
static W root[512],expected[512];static U words[128],used;
static U length(const W *p){U n=0;while(p[n])++n;return n;}static void put(U n){words[used++]=n;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static void join(W *out,const W *base,const W *tail){U i=0;while(base[i]){out[i]=base[i];++i;}U j=0;do{out[i+j]=tail[j];}while(tail[j++]);}
static void query(const W *name,U ansi){U n=length(expected);struct{U before;W value[512];U after;} w;struct{U before;char value[512];U after;} a;w.before=w.after=a.before=a.after=0x715e7a11;
 for(U i=0;i<512;++i){w.value[i]=0x7171;a.value[i]='q';}W *wp=(W *)0x71717171;char *ap=(char *)0x71717171;
 if(!ansi){put(GetFullPathNameW(name,0,0,0)==n+1);put(GetFullPathNameW(name,512,w.value,&wp)==n);U eq=1;for(U i=0;i<=n;++i)if(w.value[i]!=expected[i])eq=0;put(eq);put(wp?(U)(w.value+n-wp):0xffffffff);put(!wp||(wp>=w.value&&wp<=w.value+n));put(w.before==0x715e7a11&&w.after==0x715e7a11);}
 else{char text[512];U i=0;do{text[i]=(char)name[i];}while(name[i++]);put(GetFullPathNameA(text,0,0,0)==n+1);put(GetFullPathNameA(text,512,a.value,&ap)==n);U eq=1;for(i=0;i<=n;++i)if((unsigned char)a.value[i]!=expected[i])eq=0;put(eq);put(ap?(U)(a.value+n-ap):0xffffffff);put(!ap||(ap>=a.value&&ap<=a.value+n));put(a.before==0x715e7a11&&a.after==0x715e7a11);}
}
U entry(void){U n=GetModuleFileNameW(0,root,512);if(!n||n>=448)__builtin_trap();while(n&&root[n-1]!='\\')--n;root[n-1]=0;if(!SetCurrentDirectoryW(root))__builtin_trap();U before=stack();put(0x50415432);put(4);
 const W *names[]={L"missing.ini",L".\\folder\\..\\missing.ini",L"folder\\",L"..",L"桜.ini"};
 for(U i=0;i<5;++i){if(i==3){U at=length(root);while(at&&root[at-1]!='\\')--at;W save=root[at-1];root[at-1]=0;join(expected,root,L"");root[at-1]=save;}else join(expected,root,i==2?L"\\folder\\":i==4?L"\\桜.ini":L"\\missing.ini");query(names[i],0);if(i<4)query(names[i],1);}
 struct{U before;W out;U after;} small={0x715e7a11,0x7171,0x715e7a11};join(expected,root,L"\\missing.ini");put(GetFullPathNameW(L"missing.ini",1,&small.out,0)==length(expected)+1);put(small.out);put(small.before==0x715e7a11&&small.after==0x715e7a11);
 put(stack()==before);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;}
