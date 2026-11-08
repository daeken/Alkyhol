/* Original MIT PE32 observer. Runs unchanged on native Windows and Alkyhol. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL U32 __stdcall GetLastError(void);
DLL void __stdcall SetLastError(U32);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
extern U32 __cdecl abi_canary(void);
static U32 words[8],*volatile relocated_words=words;
U32 entry(void) {
 U32 i,written=0;
 relocated_words[0]=0x49333836u;relocated_words[1]=sizeof(void *);
 for(i=0;i<10000;i++) {SetLastError(i^0x12345678u);if(GetLastError()!=(i^0x12345678u))return 81;}
 relocated_words[2]=10000;relocated_words[3]=GetLastError();
 relocated_words[4]=abi_canary();
 relocated_words[5]=((U32)relocated_words==(U32)words);
 relocated_words[6]=0xa1b2c3d4u;relocated_words[7]=0x88776655u;
 if(!WriteFile(GetStdHandle(0xfffffff5u),words,sizeof(words),&written,0)||written!=sizeof(words))return 82;
 return 0;
}
