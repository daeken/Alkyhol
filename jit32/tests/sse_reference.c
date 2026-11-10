/* MIT. Independent original x86 assembly observation of DAO-reached scalar
 * conversion families. On Apple Silicon this executable runs under Rosetta.
 * Not a native Windows measurement or a complete SIMD equivalence claim. */
#include <stdint.h>
#include <stdio.h>
static uint64_t execute(unsigned op,uint64_t input,uint32_t before,uint32_t *after){
    uint32_t integer; uint64_t floating=0;
#define TO_INT(insn) __asm__ volatile("ldmxcsr %3; " insn " %2,%0; stmxcsr %1":"=r"(integer),"=m"(*after):"m"(input),"m"(before):"memory")
    switch(op){
    case 0: TO_INT("cvttsd2si"); return integer;
    case 1: TO_INT("cvttss2si"); return integer;
    case 2: TO_INT("cvtsd2si"); return integer;
    case 3: TO_INT("cvtss2si"); return integer;
    case 4: __asm__ volatile("ldmxcsr %3; cvtsi2sdl %2,%%xmm0; movq %%xmm0,%0; stmxcsr %1":"=m"(floating),"=m"(*after):"m"((int32_t){(int32_t)input}),"m"(before):"xmm0","memory");return floating;
    case 5: __asm__ volatile("ldmxcsr %3; cvtsi2ssl %2,%%xmm0; movd %%xmm0,%0; stmxcsr %1":"=r"(integer),"=m"(*after):"m"((int32_t){(int32_t)input}),"m"(before):"xmm0","memory");return integer;
    }
    return 0;
}
int main(void){
    uint32_t original;__asm__ volatile("stmxcsr %0":"=m"(original));
    const uint64_t doubles[]={0,UINT64_C(0x8000000000000000),UINT64_C(0x3ff8000000000000),UINT64_C(0xbff8000000000000),1,UINT64_C(0x41dfffffffc00000),UINT64_C(0xc1e0000000000000),UINT64_C(0x41e0000000000000),UINT64_C(0x7ff0000000000000),UINT64_C(0xfff0000000000000),UINT64_C(0x7ff8000000001234),UINT64_C(0x7ff0000000001234)};
    const uint64_t singles[]={0,0x80000000,0x3fc00000,0xbfc00000,1,0x4effffff,0xcf000000,0x4f000000,0x7f800000,0xff800000,0x7fc01234,0x7f801234};
    const uint64_t integers[]={0,1,0xffffffff,0x7fffffff,0x80000000,0x01000001,0xfeffffff,0x12345678};
    for(unsigned op=0;op<6;op++)for(unsigned rc=0;rc<4;rc++)for(unsigned daz=0;daz<2;daz++)for(unsigned i=0;i<(op<4?12:8);i++){
        uint64_t input=op>=4?integers[i]:(op&1?singles[i]:doubles[i]);uint32_t before=0x1f80|(rc<<13)|(daz<<6),after;
        uint64_t result=execute(op,input,before,&after);uint32_t row[]={op,before,(uint32_t)input,(uint32_t)(input>>32),(uint32_t)result,(uint32_t)(result>>32),after};
        if(fwrite(row,sizeof(row),1,stdout)!=1)return 1;
    }
    __asm__ volatile("ldmxcsr %0"::"m"(original));return 0;
}
