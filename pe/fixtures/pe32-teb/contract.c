/* Original MIT PE32 FS/TEB observer, not game-derived code. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL U32 __stdcall GetLastError(void);
DLL void __stdcall SetLastError(U32);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
static U32 read_fs(U32 offset) {
 U32 value;
 __asm__ volatile("movl %%fs:(%1),%0":"=r"(value):"r"(offset):"memory");
 return value;
}
static void write_error(U32 value) {
 __asm__ volatile("movl %0,%%fs:0x34"::"r"(value):"memory");
}
static U32 words[16];
U32 entry(void) {
 U32 i,local=0,written=0,teb=read_fs(0x18),base=read_fs(4),limit=read_fs(8);
 void *output;
 words[0]=0x42323349u;words[1]=sizeof(void *);words[2]=(teb!=0);
 words[3]=(*(volatile U32 *)(teb+0x18)==teb);
 words[4]=(base>limit);words[5]=((U32)&local<base&&(U32)&local>=limit);
 SetLastError(0x31415926u);words[6]=read_fs(0x34);
 write_error(0xa1b2c3d4u);words[7]=GetLastError();
 for(i=0;i<10000;i++) {
  SetLastError(i^0x12345678u);
  if(read_fs(0x34)!=(i^0x12345678u))return 81;
 }
 words[8]=10000;words[9]=read_fs(0x34);words[10]=GetLastError();
 words[11]=(read_fs(0x18)==teb);
 words[12]=(read_fs(4)==base);words[13]=(read_fs(8)<=((U32)&local));
 output=GetStdHandle(0xfffffff5u);
 words[14]=(read_fs(0x18)==teb);words[15]=0x88776655u;
 if(!WriteFile(output,words,sizeof(words),&written,0)||written!=sizeof(words))return 82;
 return 0;
}
