/* Owned MIT CP_UTF8 boundary observer. Query errors have no write call;
 * insufficient-buffer cases compare status/error only, not unspecified bytes. */
typedef unsigned char U8;
typedef unsigned short U16;
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL int __stdcall MultiByteToWideChar(U32,U32,const U8 *,int,U16 *,int);
DLL int __stdcall WideCharToMultiByte(U32,U32,const U16 *,int,U8 *,int,const U8 *,int *);
DLL void __stdcall SetLastError(U32);
DLL U32 __stdcall GetLastError(void);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
static const U8 byte_cases[28][12]={
 {0},{65,0,66},{65,66,67,0},{0xce,0xbb,0xf0,0x9f,0x8c,0xb8,0},
 {0xf4,0x8f,0xbf,0xbf},{0xff},{65,0xff,66},{0xf0,0x9f},
 {0xed,0xa0,0x80},{0xc0,0xaf},{0xf4,0x90,0x80,0x80},{65,0},
 {0xe0,0x80,0x80},{0xe0,0x9f,0xbf},{0xe0,0xa0,0x80},{0xed,0x9f,0xbf},
 {0xed,0xbf,0xbf},{0xf0,0x80,0x80,0x80},{0xf0,0x8f,0xbf,0xbf},{0xf0,0x90,0x80,0x80},
 {0xf4,0x8f,0xbf,0xbf},{0xf4,0xbf,0xbf,0xbf},{0xf5,0x80,0x80,0x80},{0xe1,0x80,65},
 {0xf1,0x80,0x80,65},{0xe0,0x80,65},{0xed,0xa0,65},{0xf4,0x90,65}
};
static const int byte_lengths[28]={-1,3,-1,-1,4,1,3,2,3,2,4,0,3,3,3,3,3,4,4,4,4,4,4,3,4,3,3,3};
static const U16 wide_cases[10][8]={
 {0},{65,0,66},{65,66,67,0},{0x3bb,0xd83c,0xdf38,0},{0xdbff,0xdfff},
 {0xd800},{0xdc00},{65,0xd800,66},{0xd800,0xd800,0xdc00},{65,0}
};
static const int wide_lengths[10]={-1,3,-1,-1,2,1,1,3,3,0};
static U32 output[2048],used;
static U32 *volatile relocated_output=output;
#define RECORD(v) do {relocated_output[used++]=(U32)(v);} while(0)
U32 entry(void) {
 U32 i,j,mode,written=0;U16 wide[16];U8 bytes[16];int count,result;
 RECORD(0x54584655u);RECORD(56);RECORD(20);
 for(i=0;i<28;i++) for(mode=0;mode<2;mode++) {
  U32 flags=mode?8:0,error;
  RECORD(1);RECORD(i);RECORD(flags);RECORD(byte_lengths[i]);
  for(j=0;j<16;j++)wide[j]=0xa55a;
  SetLastError(0x1234abcd);count=MultiByteToWideChar(65001,flags,byte_cases[i],byte_lengths[i],0,0);
  RECORD(count);RECORD(GetLastError());
  if(count>16)ExitProcess(71);
  if(count>0) {
   SetLastError(0x1234abcd);result=MultiByteToWideChar(65001,flags,byte_cases[i],byte_lengths[i],wide,count);error=GetLastError();
   RECORD(result);RECORD(error);
  } else {RECORD(0xffffffffu);RECORD(0xffffffffu);}
  for(j=0;j<16;j++)RECORD(wide[j]);
 }
 for(i=0;i<10;i++) for(mode=0;mode<2;mode++) {
  U32 flags=mode?128:0,error;
  RECORD(2);RECORD(i);RECORD(flags);RECORD(wide_lengths[i]);
  for(j=0;j<16;j++)bytes[j]=0xa5;
  SetLastError(0x1234abcd);count=WideCharToMultiByte(65001,flags,wide_cases[i],wide_lengths[i],0,0,0,0);
  RECORD(count);RECORD(GetLastError());
  if(count>16)ExitProcess(72);
  if(count>0) {
   SetLastError(0x1234abcd);result=WideCharToMultiByte(65001,flags,wide_cases[i],wide_lengths[i],bytes,count,0,0);error=GetLastError();
   RECORD(result);RECORD(error);
  } else {RECORD(0xffffffffu);RECORD(0xffffffffu);}
  for(j=0;j<16;j++)RECORD(bytes[j]);
 }
 SetLastError(0x1234abcd);RECORD(MultiByteToWideChar(65001,0,byte_cases[2],-1,wide,1));RECORD(GetLastError());
 SetLastError(0x1234abcd);RECORD(WideCharToMultiByte(65001,0,wide_cases[2],-1,bytes,1,0,0));RECORD(GetLastError());
 RECORD(0x454e4421u);
 if(!WriteFile(GetStdHandle(0xfffffff5u),output,used*4,&written,0)||written!=used*4)ExitProcess(73);
 return 0;
}
