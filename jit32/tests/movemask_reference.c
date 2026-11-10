/* MIT. Owned sign-mask instruction observations; no game input. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
typedef struct { uint64_t words[2]; } Vector;
#define OBSERVE(instruction) __asm__ volatile( \
    "movdqu %[src], %%xmm1; ldmxcsr %[before]; stmxcsr %[loaded]; " \
    "pushq %[initial]; popfq; movq $-1, %%rax; " instruction " %%xmm1, %%eax; " \
    "pushfq; popq %[flags]; stmxcsr %[after]; movdqu %%xmm1, %[unchanged]; ldmxcsr %[saved]" \
    : [result]"=&a"(result), [flags]"=&r"(flags), [loaded]"=m"(loaded), \
      [after]"=m"(after), [unchanged]"=m"(unchanged) \
    : [src]"m"(src), [before]"m"(before), [initial]"r"(initial), [saved]"m"(saved) \
    : "xmm1", "cc", "memory")
int main(void) {
#ifdef _WIN32
    if (_setmode(_fileno(stdout), _O_BINARY) < 0) return 4;
#endif
    uint32_t saved;
    __asm__ volatile("stmxcsr %0":"=m"(saved));
    const uint32_t controls[] = {0x1f80, 0x9fc3, 0x0040, 0x0041};
    for (unsigned kind=0; kind<3; ++kind) {
        unsigned width=kind==0?8:kind==1?32:64, lanes=128/width;
        for (unsigned mask=0; mask<(1u<<lanes); ++mask) {
            Vector src={{0,0}}, unchanged;
            for (unsigned lane=0; lane<lanes; ++lane) {
                if (width==8) {
                    unsigned char bytes[16]; memcpy(bytes,&src,16);
                    bytes[lane]=(unsigned char)(((lane*31+mask*7)&0x7f)|(((mask>>lane)&1)<<7));
                    memcpy(&src,bytes,16);
                } else if (width==32) {
                    uint32_t values[4]; memcpy(values,&src,16);
                    values[lane]=UINT32_C(0x7f800001)|(((mask>>lane)&1)<<31);
                    memcpy(&src,values,16);
                } else src.words[lane]=UINT64_C(0x7ff0000000000001)|((uint64_t)((mask>>lane)&1)<<63);
            }
            uint32_t before=controls[mask&3], loaded, after;
            uint64_t result, flags, initial=(mask&1)?0xad7:0x202;
            switch(kind) {
                case 0: OBSERVE("pmovmskb"); break;
                case 1: OBSERVE("movmskps"); break;
                case 2: OBSERVE("movmskpd"); break;
                default: return 3;
            }
            if (memcmp(&src,&unchanged,16)) return 2;
            uint32_t row[16]={kind,width,mask,before,loaded,after};
            memcpy(row+6,&src,16); memcpy(row+10,&result,8);
            row[12]=(uint32_t)initial; row[13]=(uint32_t)flags;
            row[14]=mask&3; row[15]=0x715e7a11;
            if (fwrite(row,sizeof(row),1,stdout)!=1) return 1;
        }
    }
    return 0;
}
