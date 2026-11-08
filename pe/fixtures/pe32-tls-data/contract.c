/* Original MIT compiler-generated TLS data observer (separate zero-fill-zero fixture); no game-derived code. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL U32 __stdcall GetLastError(void);
DLL void __stdcall SetLastError(U32);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
U32 _tls_index;
/* The compiler's x86 TLS ABI symbol is an absolute FS offset, not storage. */
__asm__(".globl __tls_array\n.set __tls_array,0x2c");
U32 global_value=0x13579bdfu;
#pragma section(".tls",read,write)
__declspec(allocate(".tls")) char tls_start=0;
__declspec(thread) __declspec(align(16)) U32 tls_value=0x31415926u;
__declspec(thread) U32 tls_zero;
__declspec(thread) U32 *tls_pointer=&global_value;
#pragma section(".tls$ZZZ",read,write)
__declspec(allocate(".tls$ZZZ")) char tls_end=0;
typedef void (__stdcall *Callback)(void *,U32,void *);
static Callback const callbacks[1]={0};
struct TlsDirectory {void *start,*end;U32 *index;Callback const *callbacks;U32 zero,flags;};
const struct TlsDirectory _tls_used={&tls_start,&tls_end,&_tls_index,callbacks,0,0x00500000u};
static U32 words[24];
static U32 fs_tls_array(void) {
 U32 value;__asm__ volatile("movl %%fs:0x2c,%0":"=r"(value)::"memory");return value;
}
U32 entry(void) {
 U32 written=0;U32 *array=(U32 *)fs_tls_array();
 unsigned char *block=(unsigned char *)array[_tls_index];
 U32 template_size=(U32)&tls_end-(U32)&tls_start;
 words[0]=0x54323349u;words[1]=sizeof(void *);words[2]=(array!=0);
 words[3]=(_tls_index<64);words[4]=(block!=0);
 words[5]=(((U32)block&15)==0);words[6]=(block!=(unsigned char *)&tls_start);
 words[7]=tls_value;words[8]=tls_zero;
 words[9]=(tls_pointer==&global_value);words[10]=*tls_pointer;
 words[11]=*(volatile U32 *)((U32)&tls_start+(U32)&tls_value-(U32)block);
 tls_value=0xa1b2c3d4u;tls_zero=0x88776655u;
 *tls_pointer=0x2468ace0u;
 words[12]=tls_value;words[13]=tls_zero;words[14]=global_value;
 words[15]=(tls_pointer==&global_value);
 words[16]=(fs_tls_array()==(U32)array);words[17]=(array[_tls_index]==(U32)block);
 words[18]=(U32)&tls_value>=(U32)block&&(U32)&tls_value+4<=(U32)block+template_size;
 words[19]=(U32)&tls_zero>=(U32)block&&(U32)&tls_zero+4<=(U32)block+template_size;
 words[20]=(U32)&tls_pointer>=(U32)block&&(U32)&tls_pointer+4<=(U32)block+template_size;
 SetLastError(0x11223344u);words[21]=GetLastError();
 words[22]=*(volatile U32 *)((U32)&tls_start+(U32)&tls_value-(U32)block);words[23]=0x99aabbccu;
 if(!WriteFile(GetStdHandle(0xfffffff5u),words,sizeof(words),&written,0)||written!=sizeof(words))return 82;
 return 0;
}
