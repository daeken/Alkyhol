/* Owned independent x86 assembly observer for signed memory bit-string indices. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define INSN(SIZE,TYPE,OP,LOCK) __asm__ volatile(LOCK OP SIZE " %[index], %[cell]\n\tpushfq\n\tpopq %[flags]" : [cell] "+m"(*(TYPE *)(bytes+16)),[flags] "=r"(flags) : [index] "r"((TYPE)index) : "cc","memory")
#define SELECT(SIZE,TYPE,LOCK) switch(op){case 0:INSN(SIZE,TYPE,"bt","");break;case 1:INSN(SIZE,TYPE,"bts",LOCK);break;case 2:INSN(SIZE,TYPE,"btr",LOCK);break;case 3:INSN(SIZE,TYPE,"btc",LOCK);break;}
int main(void){
 for(uint32_t width=16;width<=32;width+=16){
  int32_t indices[]={-(int32_t)width-1,-(int32_t)width,-1,0,1,(int32_t)width-1,(int32_t)width,(int32_t)width+3};
  for(uint32_t op=0;op<4;++op)for(uint32_t locked=0;locked<2;++locked){
   if(locked&&op==0)continue;
   for(unsigned k=0;k<sizeof(indices)/sizeof(indices[0]);++k){
    _Alignas(16) unsigned char bytes[32],initial[32];
    for(unsigned i=0;i<32;++i)bytes[i]=(unsigned char)(i*37);
    memcpy(initial,bytes,32);int32_t index=indices[k];uint64_t flags=0;
    if(width==16){if(locked){SELECT("w",uint16_t,"lock ")}else{SELECT("w",uint16_t,"")}}
    else{if(locked){SELECT("l",uint32_t,"lock ")}else{SELECT("l",uint32_t,"")}}
    uint32_t header[]={width,op,locked,(uint32_t)index,(uint32_t)flags};
    if(fwrite(header,sizeof(header),1,stdout)!=1||fwrite(initial,32,1,stdout)!=1||fwrite(bytes,32,1,stdout)!=1)return 2;
   }
  }
 }
 return 0;
}
