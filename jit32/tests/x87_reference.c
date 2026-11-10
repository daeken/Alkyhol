/* MIT: independent x86 host observation for the reached JIT x87 operations.
 * Compile with clang -arch x86_64; execution on Apple Silicon uses Rosetta.
 * This is NOT a native Windows observation or proof of game equivalence. */
#include <stdint.h>
#include <stdio.h>
int main(void) {
    for(unsigned precision=0;precision<2;precision++) {
        uint16_t cw=precision?0x37f:0x27f,sw;
        int64_t value=INT64_C(9007199254740992);
        unsigned char result[10];
        __asm__ volatile("fninit; fldcw %2; fildll %3; fld1; faddp; fstpt %0; fnstsw %1"
            : "=m"(result),"=m"(sw):"m"(cw),"m"(value):"st");
        printf("precision=%04x result=",cw);for(unsigned i=0;i<10;i++)printf("%02x",result[i]);printf(" status=%04x\n",sw);
    }
    for(unsigned mode=0;mode<4;mode++) {
        uint16_t cw=0x37f|(mode<<10),sw;double input=-1.5;int64_t result;
        __asm__ volatile("fninit; fldcw %2; fldl %3; fistpll %0; fnstsw %1"
            :"=m"(result),"=m"(sw):"m"(cw),"m"(input):"st");
        printf("round=%04x result=%lld status=%04x\n",cw,(long long)result,sw);
    }
    for(unsigned i=0;i<2;i++) {
        struct {uint64_t sig;uint16_t exp;} a={i?UINT64_C(0x8000000000000c00):UINT64_C(0x8000000000000400),0x3fff},b={UINT64_C(0x8000000000000000),i?0xbf9b:0x3f9b};
        uint16_t cw=0x27f,sw;unsigned char result[10];
        __asm__ volatile("fninit; fldcw %2; fldt %3; fldt %4; faddp; fstpt %0; fnstsw %1":"=m"(result),"=m"(sw):"m"(cw),"m"(a),"m"(b):"st");
        printf("double-round=%u result=",i);for(unsigned j=0;j<10;j++)printf("%02x",result[j]);printf(" status=%04x\n",sw);
    }
    uint32_t angles[]={0,0x80000000,0x3f800000,0xbf800000,0x40490fdb,0x3fc90fdb,0x40c90fdb,0x3e000000,0xc0c90fdb,0x44800000,0x1e3ce508};
    for(unsigned i=0;i<sizeof(angles)/sizeof(angles[0]);i++) {
        uint32_t sine,cosine;uint16_t sw;
        __asm__ volatile("fninit; flds %3; fsincos; fstps %0; fstps %1; fnstsw %2":"=m"(cosine),"=m"(sine),"=m"(sw):"m"(angles[i]):"st");
        printf("sincos=%08x sin=%08x cos=%08x status=%04x\n",angles[i],sine,cosine,sw);
    }
    __asm__ volatile("fninit");return 0;
}
