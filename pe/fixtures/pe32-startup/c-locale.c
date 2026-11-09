/* Original MIT system-CRT C locale query/selection witness, not game code. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) char* __cdecl setlocale(int,const char*);
__declspec(dllimport) char* __cdecl _strdup(const char*);
__declspec(dllimport) void __cdecl free(void*);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
static U words[22],used;
static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U is_c(const char* n){return n&&n[0]=='C'&&n[1]==0;}
U entry(void){U before=stack();put(0x434c4f43);put(1);
 for(int category=0;category<=5;category++)put(is_c(setlocale(category,0)));
 for(int category=0;category<=5;category++)put(is_c(setlocale(category,"C")));
 put(setlocale(4,"Alkyhol_Not_A_Valid_Locale_729")==0);put(is_c(setlocale(4,0)));
 char* original=setlocale(4,0);char* copy=_strdup(original);put(is_c(copy)&&copy!=original);
 if(copy){copy[0]='D';put(is_c(original));free(copy);}else put(0);
 char raw[]={(char)0x80,(char)0xff,0,0x7f};copy=_strdup(raw);
 put(copy&&copy!=raw&&(unsigned char)copy[0]==0x80&&(unsigned char)copy[1]==0xff&&copy[2]==0);
 if(copy)free(copy);copy=_strdup("");put(copy&&copy[0]==0);if(copy)free(copy);
 put(stack()==before);put(used+1);U done=0;
 if(!WriteFile(GetStdHandle((U)-11),words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();return 0;}
