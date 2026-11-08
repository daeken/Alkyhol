/* Original MIT actual frame-handler and hardware-context continuation observer. */
#include "types.h"
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
extern void fault32(void);
extern char fault_site,resume_site;
U32 inner_frame,outer_frame,after_eax,after_ebx,after_flags;
static U32 callbacks,order,errors,observed[32];
#ifndef NEGATIVE
#define NEGATIVE 0
#endif
extern U32 __safe_se_handler_table[];
extern char __safe_se_handler_count;
typedef struct { U32 size,unused[15],table,count; } Config;
const Config _load_config_used={72,{0},(U32)__safe_se_handler_table,(U32)&__safe_se_handler_count};
static void observe(Record32 *r,U32 frame,Context32 *c,U32 expected) {
    U32 fs;
    __asm__ volatile("movw %%fs,%%ax;movzwl %%ax,%0":"=r"(fs)::"eax");
    if(r->code!=0xc000001d||r->flags||r->nested||r->address!=(U32)&fault_site||r->count
       ||frame!=expected||c->eip!=(U32)&fault_site||c->esp!=inner_frame
       ||(c->flags&0x10007)!=0x10007||c->eax!=0x12345678||c->ebx!=0x23456789
       ||c->ecx!=0x3456789a||c->edx!=0x456789ab||c->esi!=0x56789abc
       ||c->edi!=0x6789abcd||c->ebp!=0x789abcde||c->fs!=fs)++errors;
    observed[5]=r->code;observed[6]=r->flags;observed[7]=r->count;
    observed[8]=r->address==(U32)&fault_site;observed[9]=c->esp==inner_frame;
    observed[10]=(c->flags&0x10007)==0x10007;observed[11]=c->eip==(U32)&fault_site;
    observed[12]=c->eax;observed[13]=c->ebx;observed[14]=c->ecx;observed[15]=c->edx;
    observed[16]=c->esi;observed[17]=c->edi;observed[18]=c->ebp;
    observed[19]=(c->cs&3)==3;observed[20]=c->fs==fs;
}
int __cdecl search_handler(Record32 *r,U32 frame,Context32 *c,void *dispatcher) {
    (void)dispatcher;++callbacks;order=(order<<8)|1;
    observe(r,frame,c,inner_frame);
    return 1; /* ExceptionContinueSearch, not filter-expression -1. */
}
int __cdecl continue_handler(Record32 *r,U32 frame,Context32 *c,void *dispatcher) {
    (void)dispatcher;++callbacks;order=(order<<8)|2;
    observe(r,frame,c,outer_frame);
#if NEGATIVE==1
    SetLastError(7); /* Explicitly unsupported host calls from this profile's handlers. */
#elif NEGATIVE==2
    c->floating[0]=1; /* Outside the selected integer/control/segment context. */
#elif NEGATIVE==3
    __asm__ volatile("ud2"); /* Nested hardware exceptions are unsupported. */
#endif
    c->eax=0xaabbccdd;c->ebx=0x0f1e2d3c;c->eip=(U32)&resume_site;c->eflags|=1;
    __asm__ volatile("movl $0xfeedbeef,%%fs:0x34":::"memory");
    return 0; /* ExceptionContinueExecution. */
}
int __cdecl unregistered_handler(Record32 *r,U32 frame,Context32 *c,void *dispatcher) {
    (void)r;(void)frame;(void)dispatcher;
    c->eip=(U32)&resume_site;
    return 0;
}
U32 entry(void) {
    U32 before,after,fs_self;
    SetLastError(0x11223344);
    __asm__ volatile("movl %%fs:0,%0":"=r"(before));
    fault32();
    __asm__ volatile("movl %%fs:0,%0":"=r"(after));
    __asm__ volatile("movl %%fs:0x18,%0":"=r"(fs_self));
    observed[0]=0x45323349;observed[1]=sizeof(void *);observed[2]=callbacks;observed[3]=order;observed[4]=errors;
    observed[21]=after_eax;observed[22]=after_ebx;observed[23]=after_flags&1;
    observed[24]=before==after;observed[25]=fs_self!=0;observed[26]=GetLastError();
    observed[27]=_load_config_used.size;observed[28]=_load_config_used.count;
    observed[29]=sizeof(Context32);observed[30]=sizeof(Record32);observed[31]=0x11223344;
    U32 written=0;
    return !WriteFile(GetStdHandle((U32)-11),observed,sizeof(observed),&written,0)||written!=sizeof(observed)||errors||callbacks!=2;
}
