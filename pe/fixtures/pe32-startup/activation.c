/* Original MIT observer for the actual DAO CRT activation-context blocker.
 * Native-only API observations; it is never passed off as host implementation.
 * Inputs/resources are owned and contain no game-derived manifest or bytes. */
typedef unsigned int U32;
typedef unsigned short W;
typedef void *H;
struct Context { U32 size,flags;const W *source;W arch,language;const W *directory,*resource,*application;H module; };
struct Keyed { U32 size,version;H data;U32 length;H global;U32 global_length;H base;U32 total_length;H context;U32 roster,flags;H metadata[5]; };
_Static_assert(sizeof(struct Context)==32,"Win32 ACTCTXW size");
_Static_assert(sizeof(struct Keyed)==64,"Win32 keyed-data size");
__declspec(dllimport) H __stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U32,U32 *,H);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) H __stdcall GetModuleHandleW(const W *);
__declspec(dllimport) H __stdcall FindResourceExW(H,const W *,const W *,W);
__declspec(dllimport) H __stdcall LoadResource(H,H);
__declspec(dllimport) H __stdcall LockResource(H);
__declspec(dllimport) U32 __stdcall SizeofResource(H,H);
__declspec(dllimport) H __stdcall CreateActCtxW(const struct Context *);
__declspec(dllimport) int __stdcall ActivateActCtx(H,U32 *);
__declspec(dllimport) int __stdcall DeactivateActCtx(U32,U32);
__declspec(dllimport) void __stdcall ReleaseActCtx(H);
__declspec(dllimport) int __stdcall GetCurrentActCtx(H *);
__declspec(dllimport) int __stdcall FindActCtxSectionStringW(U32,const void *,U32,const W *,struct Keyed *);
static const W file[]={ 'a','l','k','y','h','o','l','-','o','w','n','e','d','-','c','o','n','t','e','x','t','.','d','l','l',0 };
static const W missing[]={ 'a','l','k','y','h','o','l','-','m','i','s','s','i','n','g','.','d','l','l',0 };
static void emit(const void *data,U32 bytes) {
    U32 written=0;if(!WriteFile(GetStdHandle((U32)-11),data,bytes,&written,0)||written!=bytes)__builtin_trap();
}
static void resource(H module,const W *type,const W *name,W language) {
    H entry=FindResourceExW(module,type,name,language);if(!entry)__builtin_trap();
    U32 size=SizeofResource(module,entry);H data=LockResource(LoadResource(module,entry));
    if(!data||!size)__builtin_trap();emit(&size,4);emit(data,size);
}
static void query(U32 stage,U32 size,U32 flags,U32 section,const W *key,H expected) {
    struct {struct Keyed data;U32 guard;} record;
    U32 *raw=(U32 *)&record;
    for(U32 i=0;i<sizeof(record)/4;i++)raw[i]=0x715e7a11;
    record.data.size=size;
    SetLastError(0xface1234);
    U32 result=(U32)FindActCtxSectionStringW(flags,0,section,key,&record.data),error=GetLastError();
    /* Preserve all defined scalar output fields, presence/ownership of pointers,
     * the 40-byte legacy tail, and guard. Do not compare randomized addresses. */
    U32 words[20]={stage,size,flags,section,result,error,record.data.size,record.data.version,
        record.data.length,record.data.global_length,record.data.total_length,
        record.data.roster,record.data.flags,record.data.data!=0,record.data.base!=0,
        record.data.context==expected,record.data.context!=0,record.guard,0,0};
    U32 unchanged=1;for(U32 i=1;i<16;i++)if(raw[i]!=0x715e7a11)unchanged=0;
    words[18]=unchanged;
    U32 tail=1;for(U32 i=10;i<16;i++)if(raw[i]!=0x715e7a11)tail=0;
    words[19]=tail;emit(words,sizeof(words));
    if(result && (flags&1) && record.data.context)ReleaseActCtx(record.data.context);
}
U32 entry(void) {
    H module=GetModuleHandleW(0);
    const W unicode[]={0x3bb,0xd83c,0xdf38,0};
    resource(module,(const W *)24,(const W *)1,1033);
    resource(module,(const W *)24,(const W *)2,1033);
    resource(module,(const W *)10,unicode,1041);
    U32 header[4]={0x41433332,sizeof(struct Context),sizeof(struct Keyed),sizeof(H)};emit(header,sizeof(header));
    query(0,64,0,2,missing,0);query(0,40,0,2,missing,0);
    struct Context input={sizeof(input),0x88,0,0,0,0,(const W *)2,0,module};
    SetLastError(0xface1234);H context=CreateActCtxW(&input);
    U32 created[4]={1,context!=(H)-1,GetLastError(),0};
    if(context==(H)-1){emit(created,sizeof(created));return 1;}
    U32 cookie=0;SetLastError(0xface1234);created[3]=(U32)ActivateActCtx(context,&cookie);
    emit(created,sizeof(created));if(!created[3]){ReleaseActCtx(context);return 2;}
    H active=0;U32 current[3]={2,(U32)GetCurrentActCtx(&active),0};current[2]=active==context;
    emit(current,sizeof(current));if(active)ReleaseActCtx(active);
    query(1,64,0,2,file,context);query(1,40,0,2,file,context);query(1,64,1,2,file,context);
    query(1,64,0,2,missing,context);query(1,64,0,1,file,context);
    SetLastError(0xface1234);U32 done[3]={3,(U32)DeactivateActCtx(0,cookie),0};done[2]=GetLastError();emit(done,sizeof(done));
    ReleaseActCtx(context);query(2,64,0,2,file,0);
    return done[1]?0:3;
}
