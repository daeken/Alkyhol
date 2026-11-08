/* Original MIT PE32 executable-export observer. No game-derived code/data. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL U32 __stdcall GetLastError(void);
DLL void __stdcall SetLastError(U32);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL void *__stdcall GetModuleHandleA(const char *);
DLL void *__stdcall GetModuleHandleW(const unsigned short *);
DLL void *__stdcall GetProcAddress(void *,const char *);
extern unsigned char __ImageBase;
U32 export_data=0x31415926u,export_bss;
U32 __stdcall Transform(U32 a,U32 b) {return (a*33u)^b^export_data;}
U32 __stdcall Hidden(U32 value) {return value+0x12345678u;}
typedef U32 (__stdcall *Binary)(U32,U32);
typedef U32 (__stdcall *Unary)(U32);
static U32 words[26];
U32 entry(void) {
 U32 written=0;void *module,*wide,*named,*alias,*ordinal,*hidden;U32 *data,*bss;
 words[0]=0x45323349u;words[1]=sizeof(void *);
 SetLastError(0x1234abcdu);module=GetModuleHandleA(0);
 words[2]=(module==&__ImageBase);words[3]=GetLastError();
 SetLastError(0x27182818u);wide=GetModuleHandleW(0);
 words[4]=(wide==module);words[5]=GetLastError();
 SetLastError(0x11223344u);named=GetProcAddress(module,"Transform");
 words[6]=(named==(void *)&Transform);words[7]=GetLastError();
 alias=GetProcAddress(module,"Alias");ordinal=GetProcAddress(module,(const char *)17);
 words[8]=(alias==named);words[9]=(ordinal==named);
 hidden=GetProcAddress(module,(const char *)20);
 data=(U32 *)GetProcAddress(module,"Data");bss=(U32 *)GetProcAddress(module,"Bss");
 if(!named||!hidden||!data||!bss)return 81;
 words[10]=((Binary)named)(7,11);words[11]=((Unary)hidden)(9);
 words[12]=(data==&export_data);words[13]=*data;words[14]=*bss;
 *bss=0xa1b2c3d4u;words[15]=export_bss;
 SetLastError(0x55667788u);words[16]=(GetProcAddress(module,"missing_export")==0);words[17]=GetLastError();
 SetLastError(0x55667788u);words[18]=(GetProcAddress(module,"transform")==0);words[19]=GetLastError();
 SetLastError(0x55667788u);words[20]=(GetProcAddress(module,(const char *)16)==0);words[21]=GetLastError();
 SetLastError(0x55667788u);words[22]=(GetProcAddress(module,(const char *)22)==0);words[23]=GetLastError();
 SetLastError(0x55667788u);words[24]=(GetProcAddress(module,"Hidden")==0);words[25]=GetLastError();
 if(!WriteFile(GetStdHandle(0xfffffff5u),words,sizeof(words),&written,0)||written!=sizeof(words))return 82;
 return 0;
}
