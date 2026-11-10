/* MIT. Owned FPATAN observations; no game inputs or host libm reference. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
#define XMM_CLOBBERS "xmm0","xmm1","xmm2","xmm3","xmm4","xmm5","xmm6","xmm7","xmm8","xmm9","xmm10","xmm11","xmm12","xmm13","xmm14","xmm15"
int main(void) {
#ifdef _WIN32
 if(_setmode(_fileno(stdout),_O_BINARY)<0)return 4;
#endif
 const uint64_t sig[]={0,0,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000000ULL,0xc000000000000000ULL,0xc000000000000000ULL,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000000ULL,0x8000000000000001ULL,0xffffffffffffffffULL};
 const uint16_t exp[]={0,0x8000,0x3fff,0xbfff,0x4000,0xc000,0x3ffe,0xbffe,0x3ffe,0xbffe,0x3f9b,0xbf9b,0x4063,0xc063,0x3fff,0xbfff};
 const uint64_t sentinel=0xa900000000000000ULL; const uint16_t sentinel_exp=0x4005;
 for(unsigned pc=0;pc<3;pc++)for(unsigned x=0;x<16;x++)for(unsigned y=0;y<16;y++) {
  unsigned char in[512] __attribute__((aligned(16)))={0},out[512] __attribute__((aligned(16)))={0};
  uint16_t cw=0x7f|((pc==0?0:pc==1?2:3)<<8);uint32_t mxcsr=0x1fa0;
  memcpy(in,&cw,2);in[4]=7;memcpy(in+24,&mxcsr,4);
  memcpy(in+32,&sig[x],8);memcpy(in+40,&exp[x],2);memcpy(in+48,&sig[y],8);memcpy(in+56,&exp[y],2);
  memcpy(in+64,&sentinel,8);memcpy(in+72,&sentinel_exp,2);
  for(unsigned i=160;i<416;i++)in[i]=(unsigned char)(i*37+11);
  __asm__ volatile("fxrstor %1; fpatan; fxsave %0; fninit":"=m"(out):"m"(in):"st","st(1)","st(2)","st(3)","st(4)","st(5)","st(6)","st(7)","memory",XMM_CLOBBERS);
  uint16_t sw;memcpy(&sw,out+2,2);
  if(memcmp(out,in,2)||out[4]!=6||(sw&0x3800)!=0x0800||memcmp(out+24,in+24,4)||memcmp(out+48,in+64,10)||memcmp(out+160,in+160,256))return 5;
  unsigned char row[64]={0};uint32_t header[]={0,cw};memcpy(row,header,8);memcpy(row+8,in+32,10);memcpy(row+18,in+48,10);memcpy(row+28,out+32,10);memcpy(row+38,out+2,2);memcpy(row+40,out,2);row[42]=out[4];memcpy(row+44,out+24,4);memcpy(row+48,out+48,10);
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
