/* MIT. Independent owned masked x87 arithmetic observations on x86/Rosetta. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
#define XMM_CLOBBERS "xmm0","xmm1","xmm2","xmm3","xmm4","xmm5","xmm6","xmm7","xmm8","xmm9","xmm10","xmm11","xmm12","xmm13","xmm14","xmm15"
#define OBSERVE(insn) __asm__ volatile("fxrstor %1; " insn " %%st(1), %%st; fxsave %0; fninit":"=m"(out):"m"(in):"st","st(1)","memory",XMM_CLOBBERS)
int main(void) {
#ifdef _WIN32
 if(_setmode(_fileno(stdout),_O_BINARY)<0)return 4;
#endif
 const uint64_t sig[]={0,0,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000001ULL,0xffffffffffffffffULL,0x8000000000000000ULL,1,0x7fffffffffffffffULL,0x8000000000000000ULL,0x8000000000000000ULL,0xc000000000000001ULL,0xc000000000000002ULL,0xc000000000000002ULL,0x8000000000000001ULL,0x8000000000000002ULL,0x8000000000000002ULL,0xc000000000000000ULL,0xaaaaaaaaaaaaaaaaULL,0xffffffffffffffffULL};
 const uint16_t exp[]={0,0x8000,0x3fff,0xbfff,0x3fff,0x7ffe,1,0,0,0x7fff,0xffff,0x7fff,0x7fff,0xffff,0x7fff,0x7fff,0xffff,0x3ffe,0x3ffe,0x0001};
 for(unsigned op=0;op<4;op++)for(unsigned pc=0;pc<3;pc++)for(unsigned rc=0;rc<4;rc++)for(unsigned a=0;a<20;a++)for(unsigned b=0;b<20;b++) {
  unsigned char in[512] __attribute__((aligned(16)))={0},out[512] __attribute__((aligned(16)))={0};
  uint16_t cw=0x7f|((pc==0?0:pc==1?2:3)<<8)|(rc<<10);uint32_t mxcsr=0x1f80;
  memcpy(in,&cw,2);in[4]=3;memcpy(in+24,&mxcsr,4);memcpy(in+32,&sig[a],8);memcpy(in+40,&exp[a],2);memcpy(in+48,&sig[b],8);memcpy(in+56,&exp[b],2);
  switch(op){case 0:OBSERVE("fadd");break;case 1:OBSERVE("fmul");break;case 2:OBSERVE("fsub");break;default:OBSERVE("fdiv");break;}
  unsigned char row[40]={0};uint32_t header[]={op,cw};memcpy(row,header,8);memcpy(row+8,in+32,10);memcpy(row+18,in+48,10);memcpy(row+28,out+32,10);memcpy(row+38,out+2,2);
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
