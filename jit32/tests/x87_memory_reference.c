/* MIT. Owned memory-operand x87 exception-priority observations. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
#define XMM_CLOBBERS "xmm0","xmm1","xmm2","xmm3","xmm4","xmm5","xmm6","xmm7","xmm8","xmm9","xmm10","xmm11","xmm12","xmm13","xmm14","xmm15"
#define OBSERVE(insn) __asm__ volatile("fxrstor %1; " insn " %2; fxsave %0; fninit":"=m"(out):"m"(in),"m"(operand):"st","memory",XMM_CLOBBERS)
int main(void) {
#ifdef _WIN32
 if(_setmode(_fileno(stdout),_O_BINARY)<0)return 4;
#endif
 const uint64_t sig[]={0x8000000000000000ULL,0xc000000000000001ULL,0x8000000000001000ULL,1,0};
 const uint16_t exp[]={0x3fff,0x7fff,0x7fff,0,0};
 const uint64_t src[]={0,1,0x3ff0000000000000ULL,0x7ff8000000000002ULL,0x7ff0000000000002ULL,0x7ff0000000000000ULL};
 for(unsigned op=0;op<4;op++)for(unsigned pc=0;pc<3;pc++)for(unsigned rc=0;rc<4;rc++)for(unsigned a=0;a<5;a++)for(unsigned b=0;b<6;b++) {
  unsigned char in[512] __attribute__((aligned(16)))={0},out[512] __attribute__((aligned(16)))={0};
  uint16_t cw=0x7f|((pc==0?0:pc==1?2:3)<<8)|(rc<<10);uint32_t mxcsr=0x1f80;uint64_t operand=src[b];
  memcpy(in,&cw,2);in[4]=1;memcpy(in+24,&mxcsr,4);memcpy(in+32,&sig[a],8);memcpy(in+40,&exp[a],2);
  switch(op){case 0:OBSERVE("faddl");break;case 1:OBSERVE("fmull");break;case 2:OBSERVE("fsubl");break;default:OBSERVE("fdivl");break;}
  unsigned char row[40]={0};uint32_t header[]={op,cw};memcpy(row,header,8);memcpy(row+8,in+32,10);memcpy(row+18,&operand,8);memcpy(row+26,out+32,10);memcpy(row+36,out+2,2);
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
