/* Original MIT microtests for the next two observed CRT startup calls. */
typedef unsigned int U32;
struct FileTime { U32 low,high; };
_Static_assert(sizeof(struct FileTime)==8,"Win32 FILETIME width");
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
__declspec(dllimport) void __stdcall GetSystemTimeAsFileTime(struct FileTime *);
__declspec(dllimport) void *__stdcall GetProcessHeap(void);
__declspec(dllimport) void *__stdcall HeapCreate(U32,U32,U32);
__declspec(dllimport) void *__stdcall HeapAlloc(void *,U32,U32);
__declspec(dllimport) int __stdcall HeapFree(void *,U32,void *);
__declspec(dllimport) U32 __stdcall HeapSize(void *,U32,const void *);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) U32 __stdcall GetLastError(void);
static void emit(const U32 *data,U32 n) {
    U32 written=0;if(!WriteFile(GetStdHandle((U32)-11),data,n*4,&written,0)||written!=n*4)__builtin_trap();
}
U32 entry(void) {
#if MODE==0
    struct {struct FileTime time;U32 guard;} value;value.guard=0x715e7a11;
    GetSystemTimeAsFileTime(&value.time);
    void *heap=GetProcessHeap(),*private_heap=HeapCreate(0,4096,0);
    void *p=HeapAlloc(heap,0,37),*q=HeapAlloc(private_heap,0,128);
    if(!p||!q)__builtin_trap();
    U32 words[10]={0x54483332,value.time.low,value.time.high,value.guard};
    SetLastError(0xface1234);
    words[4]=HeapSize(heap,0,p);words[5]=HeapSize(private_heap,0,q);words[6]=HeapSize(private_heap,1,q);
    words[7]=GetLastError()==0xface1234;words[8]=HeapFree(heap,0,p)&&HeapFree(private_heap,0,q);words[9]=sizeof(void *);
    emit(words,10);
#else
    U32 boot=0x53543332;emit(&boot,1);
#if MODE==1
    void *heap=GetProcessHeap(),*other=HeapCreate(0,4096,0);void *p=HeapAlloc(heap,0,37);
    (void)HeapSize(other,0,p);
#else
    static const struct FileTime readonly={0,0};
    GetSystemTimeAsFileTime((struct FileTime *)&readonly);
#endif
    emit(&boot,1); /* Host restrictions only; invalid native inputs are not run. */
#endif
    return 0;
}
