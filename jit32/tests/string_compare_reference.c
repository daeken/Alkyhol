/* Owned independent x86 assembly observer: CMPS/SCAS, flags and REP endpoints. */
#include <stdint.h>
#include <stdio.h>
#define RUN(OP) __asm__ volatile("pushq %[initial]\n\tpopfq\n\t" OP "\n\tpushfq\n\tpopq %[flags]\n\tcld" : "+S"(src),"+D"(dst),"+c"(remaining),"+a"(acc),[flags]"=r"(flags) : [initial]"r"(initial) : "cc","memory")
#define MODES(OP) do {if(repeat==0){RUN(OP);}else if(repeat==1){RUN("repe " OP);}else{RUN("repne " OP);}}while(0)
int main(void){
 for(uint32_t width=8;width<=32;width*=2)for(uint32_t scan=0;scan<2;++scan)for(uint32_t repeat=0;repeat<3;++repeat)for(uint32_t backward=0;backward<2;++backward)for(uint32_t count=0;count<=5;++count)for(uint32_t sample=0;sample<2;++sample){
  unsigned char a[32],b[32];for(unsigned i=0;i<32;++i)a[i]=b[i]=(unsigned char)(i*13+7);
  b[12]^=0x80;b[18]^=1;
  if(scan){for(unsigned i=0;i<4;++i)b[8+i]=b[20+i]=(unsigned char)(0xa1b2c3d4u>>(i*8));}
  unsigned start=backward?20:8;const unsigned char *src=a+start,*dst=b+start;
  uint32_t accumulator=sample?0:0xa1b2c3d4u;uint32_t acc=accumulator;uintptr_t remaining=count;
  uint64_t flags,initial=0xad7u|(backward<<10);
  if(scan){if(width==8){MODES("scasb");}else if(width==16){MODES("scasw");}else{MODES("scasl");}}
  else{if(width==8){MODES("cmpsb");}else if(width==16){MODES("cmpsw");}else{MODES("cmpsl");}}
  uint32_t header[]={width,scan,repeat,backward,count,accumulator,(uint32_t)(src-a),(uint32_t)(dst-b),(uint32_t)remaining,(uint32_t)flags};
  if(acc!=accumulator)return 3;
  if(fwrite(header,sizeof(header),1,stdout)!=1||fwrite(a,32,1,stdout)!=1||fwrite(b,32,1,stdout)!=1)return 2;
 }
 return 0;
}
