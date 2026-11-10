/* MIT. Owned bounded RSQRT observations; normal estimates are CPU-dependent. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
typedef struct {uint32_t word[4];} Vector;
#define OBSERVE(insn) __asm__ volatile("movdqu %[old], %%xmm0; movdqu %[src], %%xmm1; ldmxcsr %[before]; pushq %[initial]; popfq; " insn " %%xmm1, %%xmm0; stmxcsr %[after]; movdqu %%xmm0, %[out]; pushfq; popq %[flags]; ldmxcsr %[saved]" : [after]"=m"(after),[out]"=m"(out),[flags]"=&r"(flags):[old]"m"(old),[src]"m"(src),[before]"m"(before),[initial]"r"(initial),[saved]"m"(saved):"xmm0","xmm1","cc","memory")
int main(void) {
#ifdef _WIN32
 if(_setmode(_fileno(stdout),_O_BINARY)<0)return 4;
#endif
 uint32_t values[1040]={0,0x80000000,1,0x7fffff,0x80000001,0xbf800000,0x7f800000,0xff800000,0x7fc01234,0x7f801234,0xffc05678,0xff805678,0x3f800000,0x40800000,0x00800000,0x7f7fffff};
 uint32_t random=0x12345678;for(unsigned i=16;i<1040;i++){random=random*1664525+1013904223;uint32_t x=random&0x7fffffff;if(x<0x00800000)x=0x00800000;if(x>0x7f7fffff)x=0x7f7fffff;values[i]=x;}
 uint32_t saved;__asm__ volatile("stmxcsr %0":"=m"(saved));
 for(uint32_t op=0;op<2;op++)for(unsigned mode=0;mode<8;mode++)for(unsigned i=0;i<1040;i++) {
  Vector old={{0x11223344,0xaabbccdd,0x7fa00001,0x80000000}},src,out;for(unsigned lane=0;lane<4;lane++)src.word[lane]=values[(i+17*lane)%1040];
  uint32_t before=((mode&1)?0x1fc0:0)|((mode>>1)<<13)|((mode&2)?0x8000:0)|0x21,after;uint64_t initial=0xad7,flags;
  if(op) {OBSERVE("rsqrtps");}else {OBSERVE("rsqrtss");}
  uint32_t row[18]={op,before,after,(uint32_t)flags};memcpy(row+4,&old,16);memcpy(row+8,&src,16);memcpy(row+12,&out,16);row[16]=(uint32_t)initial;
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
