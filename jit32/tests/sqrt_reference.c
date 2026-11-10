/* MIT. Owned legacy SIMD square-root observations; no game input. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
typedef struct {uint64_t word[2];} Vector;
#define OBSERVE(insn) __asm__ volatile("movdqu %[old], %%xmm0; movdqu %[src], %%xmm1; ldmxcsr %[before]; stmxcsr %[loaded]; pushq %[initial]; popfq; " insn " %%xmm1, %%xmm0; stmxcsr %[after]; movdqu %%xmm0, %[out]; movdqu %%xmm1, %[source_after]; pushfq; popq %[flags]; ldmxcsr %[saved]" : [loaded]"=m"(loaded),[source_after]"=m"(source_after),[after]"=m"(after),[out]"=m"(out),[flags]"=&r"(flags):[old]"m"(old),[src]"m"(src),[before]"m"(before),[initial]"r"(initial),[saved]"m"(saved):"xmm0","xmm1","cc","memory")
static uint64_t random_state=UINT64_C(0x192837465a6b7c8d);
static uint64_t random_bits(void){random_state=random_state*UINT64_C(6364136223846793005)+UINT64_C(1442695040888963407);return random_state;}
int main(void){
#ifdef _WIN32
 if(_setmode(_fileno(stdout),_O_BINARY)<0)return 4;
#endif
 const uint32_t single[]={0,0x80000000,0x3f800000,0xbf800000,0x7f800000,0xff800000,0x7fc01234,0x7f801234,1,0x7fffff,0x800000,0x40000000,0xc0000000,0x7f7fffff,0x3f7fffff,0xffc05678,0x80000001,0x807fffff,0xff801234,0x40800000,0x41100000,0x41800000,0x41c80000,0x3f800001,0x407fffff,0x40800001,0x800001,0x1000000,0x7f000000,2,4,0x7fffffff};
 const uint64_t wide[]={0,UINT64_C(0x8000000000000000),UINT64_C(0x3ff0000000000000),UINT64_C(0xbff0000000000000),UINT64_C(0x7ff0000000000000),UINT64_C(0xfff0000000000000),UINT64_C(0x7ff8000000001234),UINT64_C(0x7ff0000000001234),1,UINT64_C(0xfffffffffffff),UINT64_C(0x10000000000000),UINT64_C(0x4000000000000000),UINT64_C(0xc000000000000000),UINT64_C(0x7fefffffffffffff),UINT64_C(0x3fefffffffffffff),UINT64_C(0xfff8000000005678),UINT64_C(0x8000000000000001),UINT64_C(0x800fffffffffffff),UINT64_C(0xfff0000000001234),UINT64_C(0x4010000000000000),UINT64_C(0x4022000000000000),UINT64_C(0x4030000000000000),UINT64_C(0x4039000000000000),UINT64_C(0x3ff0000000000001),UINT64_C(0x400fffffffffffff),UINT64_C(0x4010000000000001),UINT64_C(0x10000000000001),UINT64_C(0x20000000000000),UINT64_C(0x7fe0000000000000),2,4,UINT64_C(0x7fffffffffffffff)};

 Vector left[4][512],right[4][512];
 for(unsigned form=0;form<4;form++)for(unsigned i=0;i<512;i++){
  if(form&1){for(unsigned lane=0;lane<2;lane++){left[form][i].word[lane]=i<256?wide[((i/16)+3*lane)%16]:random_bits();right[form][i].word[lane]=i<256?wide[((i%32)+5*lane)%32]:random_bits();}}
  else{uint32_t x[4],y[4];for(unsigned lane=0;lane<4;lane++){x[lane]=i<256?single[((i/16)+3*lane)%16]:(uint32_t)random_bits();y[lane]=i<256?single[((i%32)+5*lane)%32]:(uint32_t)random_bits();}memcpy(&left[form][i],x,16);memcpy(&right[form][i],y,16);}
 }
 uint32_t saved;__asm__ volatile("stmxcsr %0":"=m"(saved));
 for(unsigned kind=0;kind<4;kind++)for(unsigned mode=0;mode<16;mode++)for(unsigned i=0;i<512;i++){
  Vector old=left[kind][i],src=right[kind][i],out,source_after;
  uint32_t before=0x1f80|((mode&3)<<13)|((mode&4)?0x40:0)|((mode&8)?0x8000:0)|((i&1)?0x24:0),loaded,after;uint64_t initial=0xad7,flags;
  switch(kind){case 0:OBSERVE("sqrtss");break;case 1:OBSERVE("sqrtsd");break;case 2:OBSERVE("sqrtps");break;case 3:OBSERVE("sqrtpd");break;default:return 3;}
  if(memcmp(&src,&source_after,16))return 2;
  uint32_t row[20]={kind,before,loaded,after};memcpy(row+4,&old,16);memcpy(row+8,&src,16);memcpy(row+12,&out,16);row[16]=(uint32_t)initial;row[17]=(uint32_t)flags;row[18]=i;row[19]=mode;
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
