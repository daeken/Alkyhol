/* Owned MIT native DLL observer: original static table pointers are visible
 * inside dynamic ATTACH and DETACH, including failed-initializer cleanup. */
typedef unsigned int U32;
typedef unsigned char U8;
typedef unsigned long long U64;
typedef struct RuntimeFunction {U32 begin,end,unwind;} RuntimeFunction;
#define DLL __declspec(dllimport)
DLL RuntimeFunction *__stdcall RtlLookupFunctionEntry(U64,U64 *,void *);
DLL U8 __stdcall RtlDeleteFunctionTable(RuntimeFunction *);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
extern U8 __ImageBase,DllMain[],Probe[];
static U32 calls,trace[10];
static U32 *volatile relocated_trace=trace;
static U32 original_record(U64 pc,U64 base,RuntimeFunction *table,U32 count) {
 U32 i;U64 returned=0;RuntimeFunction *row=RtlLookupFunctionEntry(pc,&returned,0);
 for(i=0;i<count;i++)if(base+table[i].begin==pc)return row==&table[i]&&returned==base;
 return 0;
}
int lifecycle(void *module,U32 reason,void *reserved) {
 U32 pe,rva,size,written=0;U64 base=(U64)&__ImageBase,returned=0;
 RuntimeFunction *table,*row;
 pe=*(U32 *)(base+0x3c);rva=*(U32 *)(base+pe+24+112+3*8);size=*(U32 *)(base+pe+24+116+3*8);
 table=(RuntimeFunction *)(base+rva);row=RtlLookupFunctionEntry((U64)DllMain,&returned,0);
 relocated_trace[0]=0x44535441u;relocated_trace[1]=reason;relocated_trace[2]=++calls;
 relocated_trace[3]=reserved==0;relocated_trace[4]=(U64)module==base;relocated_trace[5]=size/12;
 relocated_trace[6]=original_record((U64)DllMain,base,table,size/12);
 relocated_trace[7]=original_record((U64)Probe,base,table,size/12);
 relocated_trace[8]=RtlDeleteFunctionTable(row);
 relocated_trace[9]=original_record((U64)DllMain,base,table,size/12);
 if(!WriteFile(GetStdHandle(0xfffffff5u),trace,sizeof(trace),&written,0)||written!=sizeof(trace))ExitProcess(73);
 return !(FAIL_ATTACH && reason==1);
}
