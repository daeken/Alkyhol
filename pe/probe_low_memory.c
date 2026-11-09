/* Physical macOS allocator evidence, not a Windows behavior witness. */
#include <assert.h>
#include <mach/mach.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/mman.h>
extern int alkyhol_pe32_allocate_hole(uint64_t,uint64_t,uint64_t,uint64_t,uint64_t*);
int main(void) {
    uint64_t first=0,second=0,reused=0;
    int status=alkyhol_pe32_allocate_hole(0x10000,0x7fff0000,3*65536,65536,&first);
    fprintf(stderr,"initial Mach status=%d address=%llx\n",status,(unsigned long long)first);
    assert(status==KERN_SUCCESS);
    assert(first>=0x10000 && first+3*65536<=0x7fff0000 && first%65536==0);
    assert(munmap((void*)(uintptr_t)(first+65536),65536)==0);
    volatile unsigned char *left=(void*)(uintptr_t)first,*right=(void*)(uintptr_t)(first+2*65536);
    left[0]=0x5a;right[65535]=0xc3;
    assert(alkyhol_pe32_allocate_hole(first,first+3*65536,65536,65536,&second)==KERN_SUCCESS);
    assert(second==first+65536);
    assert(left[0]==0x5a && right[65535]==0xc3);
    uint64_t guard=0x12345678;
    assert(alkyhol_pe32_allocate_hole(first,first+3*65536,65536,65536,&guard)==KERN_NO_SPACE);
    assert(guard==0x12345678 && left[0]==0x5a && right[65535]==0xc3);
    assert(munmap((void*)(uintptr_t)second,65536)==0);
    assert(alkyhol_pe32_allocate_hole(first,first+3*65536,65536,65536,&reused)==KERN_SUCCESS && reused==second);
    for(uint64_t i=0;i<65536;i++)assert(((unsigned char*)(uintptr_t)reused)[i]==0);
    assert(alkyhol_pe32_allocate_hole(first,first+3*65536,4095,65536,&guard)==KERN_INVALID_ARGUMENT);
    assert(alkyhol_pe32_allocate_hole(first,first+3*65536,4096,3,&guard)==KERN_INVALID_ARGUMENT);
    assert(alkyhol_pe32_allocate_hole(0,0x7fff0000,4096,65536,&guard)==KERN_INVALID_ARGUMENT);
    assert(alkyhol_pe32_allocate_hole(first,UINT64_MAX,4096,65536,&guard)==KERN_INVALID_ARGUMENT);
    assert(alkyhol_pe32_allocate_hole(first,first+65536,131072,65536,&guard)==KERN_NO_SPACE);
    assert(guard==0x12345678 && left[0]==0x5a && right[65535]==0xc3);
    assert(mprotect((void*)(uintptr_t)reused,65536,PROT_READ)==0);
    assert(munmap((void*)(uintptr_t)first,3*65536)==0);
    puts("bounded hole search, occupied-map preservation, gap reuse, zero-fill, bounds and munmap: pass");
    return 0;
}
