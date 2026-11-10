/* MIT. Owned bounded RCP observations; normal estimates are CPU-dependent. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
typedef struct {uint32_t word[4];} Vector;
#define OBSERVE(insn) __asm__ volatile("movdqu %[old], %%xmm0; movdqu %[src], %%xmm1; ldmxcsr %[before]; stmxcsr %[loaded]; pushq %[initial]; popfq; " insn " %%xmm1, %%xmm0; stmxcsr %[after]; movdqu %%xmm0, %[out]; movdqu %%xmm1, %[src_after]; pushfq; popq %[flags]; ldmxcsr %[saved]" : [loaded]"=m"(loaded),[src_after]"=m"(src_after),[after]"=m"(after),[out]"=m"(out),[flags]"=&r"(flags):[old]"m"(old),[src]"m"(src),[before]"m"(before),[initial]"r"(initial),[saved]"m"(saved):"xmm0","xmm1","cc","memory")
int main(void) {
#ifdef _WIN32
 if(_setmode(_fileno(stdout),_O_BINARY)<0)return 4;
#endif
 uint32_t values[1056]={0,0x80000000,1,0x7fffff,0x80000001,0xbf800000,0x7f800000,0xff800000,0x7fc01234,0x7f801234,0xffc05678,0xff805678,0x3f800000,0x40800000,0x00800000,0x7f7fffff,0x7e7fe7ff,0x7e7fe800,0x7e7fe801,0x7e7fffff,0x7e800000,0x7e800001,0x7e800bff,0x7e800c00,0x7e800c01,0xfe7fe7ff,0xfe7fe800,0xfe7fe801,0xfe7fffff,0xfe800000,0xfe800001,0xfe800c01};
 uint32_t random=0x12345678;for(unsigned i=32;i<1056;i++){random=random*1664525+1013904223;uint32_t x=random&0x7fffffff;if(x<0x00800000)x=0x00800000;if(x>0x7f7fffff)x=0x7f7fffff;values[i]=x|(random&0x80000000);}
 uint32_t saved;__asm__ volatile("stmxcsr %0":"=m"(saved));
 for(uint32_t op=0;op<2;op++)for(unsigned mode=0;mode<32;mode++)for(unsigned i=0;i<1056;i++) {
  Vector old={{0x11223344,0xaabbccdd,0x7fa00001,0x80000000}},src,out,src_after;for(unsigned lane=0;lane<4;lane++)src.word[lane]=values[(i+17*lane)%1056];
  uint32_t before=((mode&16)?0x1f80:0)|((mode&3)<<13)|((mode&4)?0x40:0)|((mode&8)?0x8000:0)|0x21,after,loaded;uint64_t initial=0xad7,flags;
  if(op) {OBSERVE("rcpps");}else {OBSERVE("rcpss");}
  uint32_t row[18]={op,before,after,(uint32_t)flags};memcpy(row+4,&old,16);memcpy(row+8,&src,16);memcpy(row+12,&out,16);row[16]=(uint32_t)initial;row[17]=loaded;
  if(memcmp(&src,&src_after,sizeof src)!=0)return 2;
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
