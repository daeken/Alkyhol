/* MIT. Independent actual x86/Rosetta arithmetic observations.
 * Finite host evidence, not native Windows or complete SIMD equivalence. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct {uint64_t word[2];} Vector;
static void execute(unsigned op,unsigned form,uint32_t mxcsr,const Vector *a,const Vector *b,Vector *out,uint32_t *after){
#define OP(insn) __asm__ volatile("ldmxcsr %4; movups %2,%%xmm0; movups %3,%%xmm1; " insn " %%xmm1,%%xmm0; movups %%xmm0,%0; stmxcsr %1":"=m"(*out),"=m"(*after):"m"(*a),"m"(*b),"m"(mxcsr):"xmm0","xmm1","memory")
 switch(op*4+form){
 case 0:OP("addss");break;
 case 1:OP("addsd");break;
 case 2:OP("addps");break;
 case 3:OP("addpd");break;
 case 4:OP("subss");break;
 case 5:OP("subsd");break;
 case 6:OP("subps");break;
 case 7:OP("subpd");break;
 case 8:OP("mulss");break;
 case 9:OP("mulsd");break;
 case 10:OP("mulps");break;
 case 11:OP("mulpd");break;
 case 12:OP("divss");break;
 case 13:OP("divsd");break;
 case 14:OP("divps");break;
 case 15:OP("divpd");break;
 default:__builtin_trap();}
}
static const uint64_t pairs32[8][2]={{UINT64_C(0x3fc00000),UINT64_C(0x40000000)},{UINT64_C(0xc0200000),UINT64_C(0x3f000000)},{UINT64_C(0x3f800000),UINT64_C(0x33800000)},{UINT64_C(0xbf800000),UINT64_C(0xb3800000)},{UINT64_C(0x40490fdb),UINT64_C(0x3f317218)},{UINT64_C(0x501502f9),UINT64_C(0x2edbe6ff)},{UINT64_C(0x0),UINT64_C(0xc0000000)},{UINT64_C(0x80000000),UINT64_C(0x40000000)}};
static const uint64_t pairs64[8][2]={{UINT64_C(0x3ff8000000000000),UINT64_C(0x4000000000000000)},{UINT64_C(0xc004000000000000),UINT64_C(0x3fe0000000000000)},{UINT64_C(0x3ff0000000000000),UINT64_C(0x3e70000000000000)},{UINT64_C(0xbff0000000000000),UINT64_C(0xbe70000000000000)},{UINT64_C(0x400921fb54442d18),UINT64_C(0x3fe62e42fefa39ef)},{UINT64_C(0x4202a05f20000000),UINT64_C(0x3ddb7cdfd9d7bdbb)},{UINT64_C(0x0),UINT64_C(0xc000000000000000)},{UINT64_C(0x8000000000000000),UINT64_C(0x4000000000000000)}};
int main(void){
 uint32_t saved;__asm__ volatile("stmxcsr %0":"=m"(saved));
 for(unsigned op=0;op<4;op++)for(unsigned form=0;form<4;form++)for(unsigned rc=0;rc<4;rc++)for(unsigned controls=0;controls<2;controls++)for(unsigned item=0;item<8;item++){
  Vector a,b,out;uint32_t after,before=0x1f83|(rc<<13)|(controls?0x8040:0);
  if(form&1){a=(Vector){{pairs64[item][0],UINT64_C(0xc004000000000000)}};b=(Vector){{pairs64[item][1],UINT64_C(0x3fe0000000000000)}};}
  else{a=(Vector){{pairs32[item][0]|UINT64_C(0xc020000000000000),UINT64_C(0xc100000040800000)}};b=(Vector){{pairs32[item][1]|UINT64_C(0x3f00000000000000),UINT64_C(0xc0000000c0000000)}};}
  execute(op,form,before,&a,&b,&out,&after);
  uint32_t row[16]={op,form,before};memcpy(row+3,&a,16);memcpy(row+7,&b,16);memcpy(row+11,&out,16);row[15]=after;
  if(fwrite(row,sizeof(row),1,stdout)!=1)return 1;
 }
 __asm__ volatile("ldmxcsr %0"::"m"(saved));return 0;
}
