/* Original MIT edge cases for the observed CRT mapping/refcount APIs. */
typedef unsigned int U32;
typedef unsigned short U16;
int _fltused=0; /* Freestanding compiler FP marker; no CRT initialization. */
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) int __stdcall LCMapStringW(U32,U32,const U16 *,int,U16 *,int);
__declspec(dllimport) U32 __stdcall InterlockedDecrement(volatile U32 *);
__declspec(dllimport) int __stdcall IsProcessorFeaturePresent(U32);
static void emit(const U32 *words,U32 n) {
    U32 written=0;if(!WriteFile(GetStdHandle((U32)-11),words,n*4,&written,0)||written!=n*4)__builtin_trap();
}
U32 entry(void) {
#if MODE==0
    const U16 source[4]={'i',0xdf,0x192,0},terminated[3]={'A','B',0};
    U16 output[5]={0,0,0,0,0xbeef},tail[3],inplace[3]={'A',0xdf,0};
    U32 words[24]={0x4c434d33};
    words[1]=(U32)LCMapStringW(0,0x200,source,4,0,0);
    words[2]=(U32)LCMapStringW(0,0x200,source,4,output,4);
    for(U32 i=0;i<5;++i)words[3+i]=output[i];
    output[1]=0xbeef;words[8]=(U32)LCMapStringW(0,0x200,source,4,output,1);
    words[9]=GetLastError();words[18]=output[1]==0xbeef;
    words[10]=(U32)LCMapStringW(0,0x100,terminated,-5,tail,3);
    for(U32 i=0;i<3;++i)words[11+i]=tail[i];
    words[14]=LCMapStringW(0,0x100,inplace,3,inplace,3)==3&&inplace[0]=='a'&&inplace[1]==0xdf&&inplace[2]==0;
    volatile U32 value=0;words[15]=InterlockedDecrement(&value)==~0u&&value==~0u;
    value=0x80000000;words[16]=InterlockedDecrement(&value)==0x7fffffff&&value==0x7fffffff;
    words[17]=sizeof(void *);
    U32 a=1,b,c,d;__asm__("cpuid":"+a"(a),"=b"(b),"=c"(c),"=d"(d));
    words[19]=IsProcessorFeaturePresent(10)!=0;words[20]=(d>>26)&1;
    if(words[19]){U32 result;__asm__("pxor %%xmm0,%%xmm0; movd %%xmm0,%0":"=r"(result)::"xmm0");words[21]=result==0;}
    words[22]=IsProcessorFeaturePresent(0)==0;
    double numerator=4195835.0,denominator=3145727.0,quotient;
    __asm__("fldl %1; fdivl %2; fstpl %0":"=m"(quotient):"m"(numerator),"m"(denominator):"st");
    double error=numerator-quotient*denominator;words[23]=error> -0.001&&error<0.001;
    emit(words,24);
#else
    U32 boot=0x53543332;emit(&boot,1);
#if MODE==4
    (void)IsProcessorFeaturePresent(3);
#else
    const U16 source[2]={MODE==3?0x2603:'A',0};U16 output[2];
    (void)LCMapStringW(MODE==1?0x409:0,MODE==2?0x400:0x100,source,2,output,2);
#endif
    emit(&boot,1); /* Unsupported locales, flags and units cannot return success. */
#endif
    return 0;
}
