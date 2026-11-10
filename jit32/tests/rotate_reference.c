/* MIT. Owned narrow rotates observed with actual x86/Rosetta assembly. */
#include <stdint.h>
#include <stdio.h>
#define ROT(INSN,OPERAND,CONSTRAINT) __asm__ volatile("pushq %[initial]; popfq; " INSN " %%cl, " OPERAND "; pushfq; popq %[flags]" : [value] CONSTRAINT(result),[flags] "=r"(flags):"c"(count),[initial]"r"(initial):"cc","memory")
int main(void) {
 const uint32_t values[]={0,0xa55a8001,0x5aa57ffe,0xffffffff};
 for(uint32_t form=0;form<5;form++)for(uint32_t dir=0;dir<2;dir++)for(uint32_t n=0;n<65;n++)for(unsigned v=0;v<4;v++)for(unsigned f=0;f<2;f++) {
  uint32_t count=n==64?255:n,input=values[v],result=input;uint64_t flags,initial=f?0xad7:0x202;
  switch(form) {
  case 0:if(dir){ROT("rorb","%b[value]","+a");}else{ROT("rolb","%b[value]","+a");}break;
  case 1:if(dir){ROT("rorb","%h[value]","+a");}else{ROT("rolb","%h[value]","+a");}break;
  case 2:if(dir){ROT("rorw","%w[value]","+a");}else{ROT("rolw","%w[value]","+a");}break;
  case 3:if(dir){ROT("rorb","%[value]","+m");}else{ROT("rolb","%[value]","+m");}break;
  default:if(dir){ROT("rorw","%[value]","+m");}else{ROT("rolw","%[value]","+m");}break;
  }
  uint32_t row[]={form,dir,count,input,(uint32_t)initial,result,(uint32_t)flags};
  if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
