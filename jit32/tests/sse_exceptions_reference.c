/* MIT. Fixed IEEE edge cases for DAO-reached denormal arithmetic.
 * All reference operations execute actual x86 assembly under Rosetta.
 * Not a native Windows or physical Intel observation. */
#define main unused_normal_main
#include "sse_arithmetic_reference.c"
#undef main
static const uint64_t edges32[24][2]={{UINT64_C(0x1),UINT64_C(0x3f800000)},{UINT64_C(0x3f800000),UINT64_C(0x1)},{UINT64_C(0x7fffff),UINT64_C(0x800000)},{UINT64_C(0x800000),UINT64_C(0x7fffff)},{UINT64_C(0x800000),UINT64_C(0x3f000000)},{UINT64_C(0x800000),UINT64_C(0x40400000)},{UINT64_C(0x800000),UINT64_C(0x3f7fffff)},{UINT64_C(0x7f7fffff),UINT64_C(0x40000000)},{UINT64_C(0x0),UINT64_C(0x0)},{UINT64_C(0x3f800000),UINT64_C(0x0)},{UINT64_C(0x3f800000),UINT64_C(0x80000000)},{UINT64_C(0x7f800000),UINT64_C(0xff800000)},{UINT64_C(0x7f800000),UINT64_C(0x0)},{UINT64_C(0x0),UINT64_C(0x7f800000)},{UINT64_C(0x7f800000),UINT64_C(0x7f800000)},{UINT64_C(0x7fc01234),UINT64_C(0x7fc05678)},{UINT64_C(0x7fc05678),UINT64_C(0x7fc01234)},{UINT64_C(0x7f801234),UINT64_C(0x7fc05678)},{UINT64_C(0x7fc01234),UINT64_C(0x7f805678)},{UINT64_C(0x7f801234),UINT64_C(0x7f805678)},{UINT64_C(0x7fc01234),UINT64_C(0x1)},{UINT64_C(0x1),UINT64_C(0x7fc05678)},{UINT64_C(0x7f801234),UINT64_C(0x1)},{UINT64_C(0x1),UINT64_C(0x7f805678)}};
static const uint64_t edges64[24][2]={{UINT64_C(0x1),UINT64_C(0x3ff0000000000000)},{UINT64_C(0x3ff0000000000000),UINT64_C(0x1)},{UINT64_C(0xfffffffffffff),UINT64_C(0x10000000000000)},{UINT64_C(0x10000000000000),UINT64_C(0xfffffffffffff)},{UINT64_C(0x10000000000000),UINT64_C(0x3fe0000000000000)},{UINT64_C(0x10000000000000),UINT64_C(0x4008000000000000)},{UINT64_C(0x10000000000000),UINT64_C(0x3fefffffffffffff)},{UINT64_C(0x7fefffffffffffff),UINT64_C(0x4000000000000000)},{UINT64_C(0x0),UINT64_C(0x0)},{UINT64_C(0x3ff0000000000000),UINT64_C(0x0)},{UINT64_C(0x3ff0000000000000),UINT64_C(0x8000000000000000)},{UINT64_C(0x7ff0000000000000),UINT64_C(0xfff0000000000000)},{UINT64_C(0x7ff0000000000000),UINT64_C(0x0)},{UINT64_C(0x0),UINT64_C(0x7ff0000000000000)},{UINT64_C(0x7ff0000000000000),UINT64_C(0x7ff0000000000000)},{UINT64_C(0x7ff8000000001234),UINT64_C(0x7ff8000000005678)},{UINT64_C(0x7ff8000000005678),UINT64_C(0x7ff8000000001234)},{UINT64_C(0x7ff0000000001234),UINT64_C(0x7ff8000000005678)},{UINT64_C(0x7ff8000000001234),UINT64_C(0x7ff0000000005678)},{UINT64_C(0x7ff0000000001234),UINT64_C(0x7ff0000000005678)},{UINT64_C(0x7ff8000000001234),UINT64_C(0x1)},{UINT64_C(0x1),UINT64_C(0x7ff8000000005678)},{UINT64_C(0x7ff0000000001234),UINT64_C(0x1)},{UINT64_C(0x1),UINT64_C(0x7ff0000000005678)}};
int main(void){
 uint32_t saved;__asm__ volatile("stmxcsr %0":"=m"(saved));
 for(unsigned op=0;op<4;op++)for(unsigned form=0;form<4;form++)for(unsigned rc=0;rc<4;rc++)for(unsigned controls=0;controls<4;controls++)for(unsigned item=0;item<24;item++){
  Vector a,b,out;uint32_t after,before=0x1f80|(rc<<13)|((controls&1)?0x40:0)|((controls&2)?0x8000:0);
  if(form&1){a=(Vector){{edges64[item][0],UINT64_C(0xc004000000000000)}};b=(Vector){{edges64[item][1],UINT64_C(0x3fe0000000000000)}};}
  else{a=(Vector){{edges32[item][0]|UINT64_C(0xc020000000000000),UINT64_C(0xc100000040800000)}};b=(Vector){{edges32[item][1]|UINT64_C(0x3f00000000000000),UINT64_C(0xc0000000c0000000)}};}
  execute(op,form,before,&a,&b,&out,&after);
  uint32_t row[16]={op,form,before};memcpy(row+3,&a,16);memcpy(row+7,&b,16);memcpy(row+11,&out,16);row[15]=after;
  if(fwrite(row,sizeof(row),1,stdout)!=1)return 1;
 }
 __asm__ volatile("ldmxcsr %0"::"m"(saved));return 0;
}
