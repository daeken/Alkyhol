/* Original MIT exclusions witness. Deliberately invalid pointers are executed
 * only through Alkyhol's checked runtime, never used as native Windows tests. */
typedef unsigned int U;
typedef unsigned short W;
typedef void *H;
struct NF { U digits,zero,grouping; const W *decimal,*thousands; U negative; };
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) int __stdcall GetNumberFormatW(U,U,const W *,const struct NF *,W *,int);
U entry(void) {
    U marker[2]={0x4e464731,MODE},done=0;
    if(!WriteFile(GetStdHandle((U)-11),marker,sizeof(marker),&done,0) || done!=sizeof(marker)) __builtin_trap();
    W value[128]={49,50,51,0},output[128];
    static const W readonly[128]={49,0};
    struct NF f={2,1,3,(const W *)L".",(const W *)L",",1};
    U locale=0x409; const W *input=value; W *target=output; const struct NF *format=&f;
    (void)readonly;
#if MODE == 0
    locale=0x407;
#elif MODE == 1
    format=0;
#elif MODE == 2
    format=(const struct NF *)1;
#elif MODE == 3
    target=(W *)(void *)readonly;
#elif MODE == 4
    input=(const W *)1;
#elif MODE == 5
    target=(W *)0x1000;
#elif MODE == 6
    target=value;
#elif MODE == 7
    f.grouping=123;
#else
#error Unknown mode
#endif
    GetNumberFormatW(locale,0,input,format,target,128);
    return 99;
}
