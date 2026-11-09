/* Owned MIT instruction-feature self-consistency check for original D3DX. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) int __stdcall IsProcessorFeaturePresent(U);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
static U words[32],used;static void put(U n){words[used++]=n;}
static U stack(void){U s;__asm__ volatile("movl %%esp,%0":"=r"(s));return s;}
U entry(void){U before=stack();put(0x46555043);put(1);
 U a=1,b,c,d;__asm__ volatile("cpuid":"+a"(a),"=b"(b),"=c"(c),"=d"(d));U basic=d;
 a=0x80000000;__asm__ volatile("cpuid":"+a"(a),"=b"(b),"=c"(c),"=d"(d));U extended=0;
 if(a>=0x80000001){a=0x80000001;__asm__ volatile("cpuid":"+a"(a),"=b"(b),"=c"(c),"=d"(d));extended=d;}
 const U features[4]={3,6,7,10};U expected[4]={(basic>>23)&1,(basic>>25)&1,(extended>>31)&1,(basic>>26)&1};
 for(U n=0;n<4;n++){SetLastError(0xface1234);put((IsProcessorFeaturePresent(features[n])!=0)==expected[n]);put(GetLastError());}
 U zero=1;if(expected[3]){__asm__ volatile("pxor %%xmm0,%%xmm0;movd %%xmm0,%0":"=r"(zero)::"xmm0");}put(!expected[3]||zero==0);
 put(stack()==before);put(used+1);U done=0;return WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)&&done==used*4?0:7;
}
