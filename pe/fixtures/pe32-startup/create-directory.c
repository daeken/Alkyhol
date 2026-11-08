/* Original MIT probe for DAO's reached directory constructor. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W *,U);
__declspec(dllimport) int __stdcall CreateDirectoryW(const W *,H);
static W path[512];static U prefix,words[32],used;
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static void put(U n){words[used++]=n;}
static void append(const W *name){U i=0;do{path[prefix+i]=name[i];}while(name[i++]);}
static void create(const W *name){append(name);SetLastError(0xface1234);put(CreateDirectoryW(path,0)!=0);put(GetLastError());}
U entry(void){
    U n=GetModuleFileNameW(0,path,512);if(!n||n>=448)__builtin_trap();prefix=n;while(prefix&&path[prefix-1]!='\\')--prefix;if(!prefix)__builtin_trap();
    append(L"root");prefix+=4;
    put(0x4d4b4432);put(sizeof(H));U before=stack();(void)before;
#if MODE == 1
    append(L"\\first");(void)CreateDirectoryW(path,(H)1);__builtin_trap();
#elif MODE == 2
    append(L"-outside\\escape");(void)CreateDirectoryW(path,0);__builtin_trap();
#elif MODE == 3
    append(L"\\..\\escape");(void)CreateDirectoryW(path,0);__builtin_trap();
#elif MODE == 4
    append(L"\\linked\\escape");(void)CreateDirectoryW(path,0);__builtin_trap();
#elif MODE == 5
    append(L"\\new-桜");(void)CreateDirectoryW(path,0);__builtin_trap();
#elif MODE == 6
    (void)CreateDirectoryW(L"relative",0);__builtin_trap();
#else
    create(L"\\first");create(L"\\first");create(L"\\first\\child");create(L"\\missing\\child");
    create(L"\\file");create(L"");create(L"\\file\\child");create(L"\\first\\child\\");create(L"\\trailing\\");
    path[prefix-5]=0;SetLastError(0xface1234);put(CreateDirectoryW(path,0)!=0);put(GetLastError());
    put(stack()==before);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
}
