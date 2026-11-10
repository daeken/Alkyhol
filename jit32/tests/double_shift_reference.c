/* MIT. Owned SHLD/SHRD observations, actual x86/Rosetta assembly. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#ifdef __APPLE__
#include <signal.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>
static void fault_exit(int signal) {_exit(64+signal);}
static int write_checks(void) {
 for(unsigned op=0;op<2;op++)for(unsigned count=0;count<2;count++) {
  pid_t child=fork();if(child<0)return 2;
  if(child==0) {
   signal(SIGBUS,fault_exit);signal(SIGSEGV,fault_exit);size_t size=(size_t)sysconf(_SC_PAGESIZE);uint32_t *p=mmap(NULL,size,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANON,-1,0);if(p==MAP_FAILED)_exit(3);*p=0x12345678;if(mprotect(p,size,PROT_READ))_exit(4);
   if(op)__asm__ volatile("shrdl %%cl, %%edx, (%0)"::"r"(p),"c"(count),"d"(0xabcdef01):"cc","memory");
   else __asm__ volatile("shldl %%cl, %%edx, (%0)"::"r"(p),"c"(count),"d"(0xabcdef01):"cc","memory");
   _exit(0);
  }
  int status;if(waitpid(child,&status,0)!=child)return 5;uint32_t row[]={op,count,(uint32_t)status};if(fwrite(row,sizeof row,1,stdout)!=1)return 6;
 }
 return 0;
}
#endif
#define OBSERVE(OP,CONSTRAINT,DEST) __asm__ volatile("pushq %[initial]; popfq; " OP " %%cl, %%edx, " DEST "; pushfq; popq %[flags]" : [result] CONSTRAINT(result),[flags]"=r"(flags):"d"(source),"c"(count),[initial]"r"(initial):"cc","memory")
int main(void) {
#ifdef __APPLE__
 if(getenv("ALKYHOL_DOUBLE_SHIFT_WRITE_CHECK"))return write_checks();
#endif
 const uint32_t values[]={0,0x80000001,0x7ffffffe,0xffffffff};
 for(uint32_t op=0;op<2;op++)for(uint32_t memory=0;memory<2;memory++)for(uint32_t n=0;n<65;n++)for(unsigned a=0;a<4;a++)for(unsigned b=0;b<4;b++)for(unsigned f=0;f<2;f++) {
  uint32_t count=n==64?255:n,input=values[a],source=values[b],result=input;uint64_t initial=f?0xad7:0x202,flags;
  if(memory){if(op){OBSERVE("shrdl","+m","%[result]");}else{OBSERVE("shldl","+m","%[result]");}}
  else {if(op){OBSERVE("shrdl","+a","%k[result]");}else{OBSERVE("shldl","+a","%k[result]");}}
  uint32_t row[]={op,memory,count,input,source,(uint32_t)initial,result,(uint32_t)flags};if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
 }
 return 0;
}
