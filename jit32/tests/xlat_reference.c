/* MIT. Owned XLAT observations, actual x86/Rosetta rather than Windows. */
#include <stdint.h>
#include <stdio.h>
int main(void) {
 uint8_t table[256];for(unsigned i=0;i<256;i++)table[i]=(uint8_t)((i*73)^0xa5);
 for(uint32_t index=0;index<256;index++)for(unsigned f=0;f<2;f++) {
  uint32_t input=0xa55a8000|index,result=input;uint64_t flags,initial=f?0xad7:0x202;
  __asm__ volatile("pushq %[initial]; popfq; xlatb; pushfq; popq %[flags]" : "+a"(result),[flags]"=r"(flags):"b"(table),[initial]"r"(initial):"cc","memory");
  uint32_t row[]={input,(uint32_t)initial,result,(uint32_t)flags};if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
