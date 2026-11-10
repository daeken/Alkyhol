/* Original MIT witness for the GetNumberFormatW call reached by DAO.
 * No game code or data. The identical PE runs on Windows and both Mac hosts. */
typedef unsigned int U;
typedef unsigned short W;
typedef void *H;
struct NF { U digits, zero, grouping; const W *decimal, *thousands; U negative; };
_Static_assert(sizeof(H) == 4 && sizeof(struct NF) == 24, "Win32 NUMBERFMTW");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H, const void *, U, U *, H);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) int __stdcall GetNumberFormatW(U,U,const W *,const struct NF *,W *,int);
static void emit(const void *p, U n) {
    U done = 0;
    if (!WriteFile(GetStdHandle((U)-11), p, n, &done, 0) || done != n) __builtin_trap();
}
static U serial;
static void observe(const W *value, const struct NF *fmt, U flags, int capacity, U query) {
    struct { U index, count, error; W before, output[128], after; } row;
    _Static_assert(sizeof(row) == 272, "fixed observation row");
    row.index = serial++; row.before = 0x1234; row.after = 0x5678;
    for (U i=0;i<128;++i) row.output[i]=0xa5a5;
    SetLastError(0xabcdef01);
    row.count = (U)GetNumberFormatW(0x409,flags,value,fmt,query?0:row.output,capacity);
    row.error = GetLastError(); emit(&row,sizeof(row));
}
#define S(x) ((const W *)L##x)
U entry(void) {
    U header[2]={0x4e463332,0}; emit(header,sizeof(header));
    struct NF f={0,1,3,S("."),S(","),1};
    static const W *values[]={S("0"),S("-0"),S("0.000"),S("-0.000"),
      S("0.0049"),S("-0.0049"),S("0.005"),S("-0.005"),S("0.9995"),S("-0.9995"),
      S("9.995"),S("99.9999999995"),S("123456789012345678901234567890.5"),
      S("0000123.004500"),S("1."),S("-1."),S(".5"),S("-.5"),S(""),S("-"),S("."),
      S("-."),S("+1"),S(" 1"),S("1 "),S("1,2"),S("1e2"),S("1.2.3"),S("--1"),S("１２")};
    for(U i=0;i<sizeof(values)/sizeof(values[0]);++i)
      for(f.digits=0;f.digits<10;++f.digits) observe(values[i],&f,0,128,0);
    for(U g=0;g<11;++g) for(f.negative=0;f.negative<5;++f.negative)
      for(f.zero=0;f.zero<2;++f.zero) {
        f.digits=2; f.grouping=g==10?32:g;
        observe(S("-123456789.555"),&f,0,128,0);
        observe(S("-0.005"),&f,0,128,0);
      }
    f=(struct NF){2,1,3,S("::"),S("_"),1}; observe(S("1234567.995"),&f,0,128,0);
    f.decimal=S("\u066b"); f.thousands=S("\u202f"); observe(S("1234567.995"),&f,0,128,0);
    f.decimal=S(""); f.thousands=S(""); observe(S("1234.5"),&f,0,128,0);
    f=(struct NF){2,1,3,S("."),S(","),1};
    for(int n=0;n<=12;++n) observe(S("-1234.995"),&f,0,n,0);
    observe(S("-1234.995"),&f,0,0,1); observe(S("-1234.995"),&f,0,12,1);
    observe(S("1"),&f,1,128,0); observe(S("1"),&f,0x80000000,128,0);
    f.digits=10; observe(S("1"),&f,0,128,0); f.digits=2;
    f.zero=2; observe(S("1"),&f,0,128,0); f.zero=1;
    f.grouping=10; observe(S("1"),&f,0,128,0); f.grouping=3;
    f.negative=5; observe(S("-1"),&f,0,128,0); f.negative=1;
    f.decimal=0; observe(S("1"),&f,0,128,0); f.decimal=S(".");
    f.thousands=0; observe(S("1"),&f,0,128,0); f.thousands=S(",");
    observe(0,&f,0,128,0); observe(S("1"),&f,0,-1,0);
    U tail[2]={0x454e4432,serial}; emit(tail,sizeof(tail)); return 0;
}
