/* Original MIT. Capture actual loader-scoped contexts and complete DLL section
 * bytes. No original game data, host execution claim, or guessed lookup result. */
#include "ctx-observe.h"
static W path[1024];
static const W key[]={'c','t','x','-','p','r','o','v','i','d','e','r','.','d','l','l',0};
static void emit(const void *data,U32 bytes) {
    U32 written=0;if(!WriteFile(GetStdHandle((U32)-11),data,bytes,&written,0)||written!=bytes)__builtin_trap();
}
static void query(U32 flags,H current) {
    struct {struct Keyed data;U32 guard;} record;
    unsigned char *raw=(unsigned char *)&record;
    const unsigned char sentinel[4]={0x11,0x7a,0x5e,0x71};
    for(U32 i=0;i<sizeof(record);i++)raw[i]=sentinel[i%4];
    record.data.size=64;SetLastError(0xface1234);
    U32 result=(U32)FindActCtxSectionStringW(flags,0,2,key,&record.data),error=GetLastError();
    struct Keyed *d=&record.data;U32 offset=(U32)-1,global_offset=(U32)-1;
    if(result) {
        if(!d->base||!d->data||d->total_length>65536||d->length>4096)__builtin_trap();
        offset=(U32)d->data-(U32)d->base;
        if(offset>d->total_length||d->length>d->total_length-offset)__builtin_trap();
        if(d->global_length) {
            global_offset=(U32)d->global-(U32)d->base;
            if(!d->global||global_offset>d->total_length||d->global_length>d->total_length-global_offset)__builtin_trap();
        }
    }
    U32 unchanged=1,tail=1;
    for(U32 i=4;i<64;i++)if(raw[i]!=sentinel[i%4])unchanged=0;
    for(U32 i=40;i<64;i++)if(raw[i]!=sentinel[i%4])tail=0;
    U32 words[19]={flags,result,error,d->size,d->version,d->length,d->global_length,d->total_length,d->roster,d->flags,
        offset,d->global!=0,d->context!=0,d->context==current,tail,record.guard,unchanged,(U32)d->context,global_offset};
    emit(words,sizeof(words));
    U32 length=result?d->length:0;emit(&length,4);if(length)emit(d->data,length);
    length=result?d->total_length:0;emit(&length,4);if(length)emit(d->base,length);
    if(result&&(flags&1)&&d->context)ReleaseActCtx(d->context);
}
void observe(U32 actor,U32 reason,H module) {
    U32 previous=GetLastError();H current=0;SetLastError(0xface1234);
    U32 ok=(U32)GetCurrentActCtx(&current),error=GetLastError();
    U32 header[7]={0x43545832,actor,reason,ok,error,(U32)current,sizeof(H)};emit(header,sizeof(header));
    if(!ok)__builtin_trap();
    U32 n=GetModuleFileNameW(module,path,1024);if(!n||n>=1024||path[n]!=0)__builtin_trap();
    emit(&n,4);emit(path,(n+1)*2);
    query(0,current);query(1,current);
    if(current)ReleaseActCtx(current);
    SetLastError(previous);
}
