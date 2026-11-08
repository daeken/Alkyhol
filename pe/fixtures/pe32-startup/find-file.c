/* Original MIT exact-file observer for DAO's measured CRT metadata query.
 * No game/runtime files. Real timestamps are checked per host, not cross-host. */
typedef unsigned int U;
typedef unsigned short W;
typedef void *H;
typedef unsigned char B;
struct Data {U attr,creation[2],access[2],write[2],high,low,reserved[2];W name[260],alternate[14];};
_Static_assert(sizeof(H)==4&&sizeof(struct Data)==592,"i386 find layout");
_Static_assert(__builtin_offsetof(struct Data,name)==44&&__builtin_offsetof(struct Data,alternate)==564,"find string offsets");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetModuleFileNameW(H,W *,U);
__declspec(dllimport) H __stdcall FindFirstFileW(const W *,struct Data *);
__declspec(dllimport) int __stdcall FindNextFileW(H,struct Data *);
__declspec(dllimport) int __stdcall FindClose(H);
static W path[1024];
static struct {struct Data data;U guard;} scratch;
static struct Data first;
static U words[64],used;
static void put(U x){if(used>=64)__builtin_trap();words[used++]=x;}
static void fill(void){B *p=(B *)&scratch;for(U i=0;i<sizeof(scratch);i++)p[i]=0xa5;}
static U untouched(void){const B *p=(const B *)&scratch.data;for(U i=0;i<sizeof(scratch.data);i++)if(p[i]!=0xa5)return 0;return 1;}
static void emit(const void *p,U n){U done=0;if(!WriteFile(GetStdHandle((U)-11),p,n,&done,0)||done!=n)__builtin_trap();}
static void stage(U n){emit(&n,4);}
static void suffix(U at,const W *name){U i=0;do{if(at+i>=1024)__builtin_trap();path[at+i]=name[i];}while(name[i++]);}
U entry(void){
    static const W filename[]={'p','r','o','b','e','.','d','a','t',0};
    static const W missing[]={'a','b','s','e','n','t','.','d','a','t',0};
    static const W missing_parent[]={'a','b','s','e','n','t','-','d','i','r','\\','p','r','o','b','e','.','d','a','t',0};
    stage(0x46465354);
    U n=GetModuleFileNameW(0,path,1024);if(!n||n>=1023)__builtin_trap();stage(0x10);
    U at=n;while(at&&path[at-1]!='\\')at--;if(!at)__builtin_trap();
    suffix(at,filename);fill();SetLastError(0xface1234);
    H h=FindFirstFileW(path,&scratch.data);U error=GetLastError();
    put(0x464e3332);put(sizeof(H));put(sizeof(struct Data));put(h!=(H)-1&&h!=0);put(error);put(scratch.guard);
    if(h==(H)-1||h==0)__builtin_trap();
    stage(0x20);
    for(U i=0;i<sizeof(first);i++)((B *)&first)[i]=((B *)&scratch.data)[i];
    fill();SetLastError(0xface1234);put((U)FindNextFileW(h,&scratch.data));put(GetLastError());put(untouched());put(scratch.guard);stage(0x30);
    SetLastError(0xface1234);put((U)FindClose(h));put(GetLastError());stage(0x40);
    /* Closed handles are outside this observer's supported lifecycle. */
    suffix(at,missing);fill();SetLastError(0xface1234);put((U)FindFirstFileW(path,&scratch.data));put(GetLastError());put(untouched());put(scratch.guard);stage(0x50);
    suffix(at,missing_parent);fill();SetLastError(0xface1234);put((U)FindFirstFileW(path,&scratch.data));put(GetLastError());put(untouched());put(scratch.guard);stage(0x60);
    put(used+1);emit(words,used*4);emit(&first,sizeof(first));return 0;
}
