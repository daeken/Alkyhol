/* Owned independent actual x86 comparisons, including NaNs and denormals. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct {uint64_t word[2];} Vector;
#define ASM(OP) __asm__ volatile("movdqu %[a], %%xmm0\n\tmovdqu %[b], %%xmm1\n\tldmxcsr %[before]\n\tpushq %[initial]\n\tpopfq\n\t" OP "\n\tstmxcsr %[after]\n\tmovdqu %%xmm0, %[out]\n\tpushfq\n\tpopq %[flags]" : [out]"=m"(*out),[after]"=m"(*after),[flags]"=r"(flags) : [a]"m"(*a),[b]"m"(*b),[before]"m"(before),[initial]"r"(initial) : "xmm0","xmm1","cc","memory")
#define PREDICATES(OP) switch(pred){case 0:ASM(OP " $0, %%xmm1, %%xmm0");break;case 1:ASM(OP " $1, %%xmm1, %%xmm0");break;case 2:ASM(OP " $2, %%xmm1, %%xmm0");break;case 3:ASM(OP " $3, %%xmm1, %%xmm0");break;case 4:ASM(OP " $4, %%xmm1, %%xmm0");break;case 5:ASM(OP " $5, %%xmm1, %%xmm0");break;case 6:ASM(OP " $6, %%xmm1, %%xmm0");break;case 7:ASM(OP " $7, %%xmm1, %%xmm0");break;}
static void execute(unsigned kind,unsigned pred,uint32_t before,const Vector *a,const Vector *b,Vector *out,uint32_t *after,uint32_t *eflags){
 uint64_t flags=0,initial=0xad7;
 switch(kind){case 0:{PREDICATES("cmpss");break;}case 1:{PREDICATES("cmpsd");break;}case 2:{PREDICATES("cmpps");break;}case 3:{PREDICATES("cmppd");break;}case 4:ASM("comiss %%xmm1, %%xmm0");break;case 5:ASM("ucomiss %%xmm1, %%xmm0");break;case 6:ASM("comisd %%xmm1, %%xmm0");break;case 7:ASM("ucomisd %%xmm1, %%xmm0");break;}
 *eflags=(uint32_t)flags;
}
int main(void){
 const uint32_t single[]={0,0x80000000,0x3f800000,0xbf800000,0x7f800000,0xff800000,0x7fc01234,0x7f801234,1,0x7fffff,0x800000,0x40000000,0xc0000000,0x7f7fffff,0x3f7fffff,0xffc05678};
 const uint64_t wide[]={0,UINT64_C(0x8000000000000000),UINT64_C(0x3ff0000000000000),UINT64_C(0xbff0000000000000),UINT64_C(0x7ff0000000000000),UINT64_C(0xfff0000000000000),UINT64_C(0x7ff8000000001234),UINT64_C(0x7ff0000000001234),1,UINT64_C(0xfffffffffffff),UINT64_C(0x10000000000000),UINT64_C(0x4000000000000000),UINT64_C(0xc000000000000000),UINT64_C(0x7fefffffffffffff),UINT64_C(0x3fefffffffffffff),UINT64_C(0xfff8000000005678)};
 uint32_t saved;__asm__ volatile("stmxcsr %0":"=m"(saved));
 for(unsigned kind=0;kind<8;++kind)for(unsigned pred=0;pred<(kind<4?8:1);++pred)for(unsigned daz=0;daz<2;++daz)for(unsigned i=0;i<16;++i)for(unsigned j=0;j<16;++j){
  Vector a,b,out;if(kind==1||kind==3||kind>=6){a=(Vector){{wide[i],wide[(i+3)%16]}};b=(Vector){{wide[j],wide[(j+5)%16]}};}
  else{uint32_t x[4],y[4];for(unsigned lane=0;lane<4;++lane){x[lane]=single[(i+lane*3)%16];y[lane]=single[(j+lane*5)%16];}memcpy(&a,x,16);memcpy(&b,y,16);}
  uint32_t before=0x1f80|(daz?0x40:0)|((i&1)?0x20:0)|(((i+j)&3)<<13)|(((i^j)&1)?0x8000:0),after,flags;
  execute(kind,pred,before,&a,&b,&out,&after,&flags);
  uint32_t row[18]={kind,pred,before,0xad7};memcpy(row+4,&a,16);memcpy(row+8,&b,16);memcpy(row+12,&out,16);row[16]=after;row[17]=flags;
  if(fwrite(row,sizeof(row),1,stdout)!=1)return 2;
 }
 __asm__ volatile("ldmxcsr %0"::"m"(saved));return 0;
}
