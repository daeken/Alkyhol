/* Alkyhol MIT: SDK-owned filesystem layouts stay on the native C side. */
#include <sys/statvfs.h>
#include <stdint.h>
#include <errno.h>
int32_t alkyhol_disk_blocks(const char* path,uint64_t* blocks) {
    struct statvfs value;
    if(statvfs(path,&value)!=0)return errno;
    blocks[0]=value.f_frsize;
    blocks[1]=value.f_bavail;
    blocks[2]=value.f_blocks;
    blocks[3]=value.f_bfree;
    return 0;
}
