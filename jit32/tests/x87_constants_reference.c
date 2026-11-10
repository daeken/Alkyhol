/* MIT. Independent x86/Rosetta observations, not native Windows evidence. */
#include <stdint.h>
#include <stdio.h>
int main(void){
 for(unsigned op=0;op<7;op++)for(unsigned rc=0;rc<4;rc++)for(unsigned pc=0;pc<3;pc++){
  uint16_t control=0x7f|((pc==0?0:pc==1?2:3)<<8)|(rc<<10),status;unsigned char value[10];
#define LOAD(insn) __asm__ volatile("fninit; fldcw %2; " insn "; fnstsw %1; fstpt %0":"=m"(value),"=m"(status):"m"(control):"st")
  switch(op){case 0:LOAD("fldpi");break;case 1:LOAD("fldl2t");break;case 2:LOAD("fldl2e");break;case 3:LOAD("fldlg2");break;case 4:LOAD("fldln2");break;case 5:LOAD("fld1");break;case 6:LOAD("fldz");break;}
  uint32_t head[]={op,control,status};if(fwrite(head,sizeof(head),1,stdout)!=1||fwrite(value,10,1,stdout)!=1)return 1;
 }
 __asm__ volatile("fninit");return 0;
}
