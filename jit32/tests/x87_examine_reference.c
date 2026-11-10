/* MIT. Owned FXAM observations on actual x86/Rosetta, not Windows evidence. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
 const uint64_t sig[] = {0,1,0x7fffffffffffffffULL,0x8000000000000000ULL,0x8000000000000000ULL,0xffffffffffffffffULL,0x8000000000000000ULL,0xc000000000000001ULL,0x8000000000000001ULL,1,0,0x8000000000000000ULL};
 const uint16_t exp[] = {0,0,0,0,0x3fff,0x7ffe,0x7fff,0x7fff,0x7fff,1,0x7fff,1};
 for(unsigned kind=0;kind<12;kind++)for(unsigned sign=0;sign<2;sign++)for(unsigned empty=0;empty<2;empty++)for(unsigned top=0;top<8;top++)for(unsigned sticky=0;sticky<2;sticky++) {
  unsigned char in[512] __attribute__((aligned(16)))={0},out[512] __attribute__((aligned(16)))={0};
  uint16_t cw=0x37f,sw=(top<<11)|(sticky?0x477f:0),se=exp[kind]|(sign<<15);uint32_t mxcsr=0x1f80;
  memcpy(in,&cw,2);memcpy(in+2,&sw,2);in[4]=empty?0:(1<<top);memcpy(in+24,&mxcsr,4);memcpy(in+32,&sig[kind],8);memcpy(in+40,&se,2);
  __asm__ volatile("fxrstor %1; fxam; fxsave %0; fninit":"=m"(out):"m"(in):"st","memory");
  unsigned char row[32]={0};memcpy(row,in+32,10);row[10]=top;row[11]=empty;memcpy(row+12,&sw,2);memcpy(row+14,out+2,2);memcpy(row+16,out+32,10);memcpy(row+26,out,2);row[28]=out[4];
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
