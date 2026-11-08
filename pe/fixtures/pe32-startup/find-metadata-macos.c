/* Original MIT host-only independent SDK stat reader. No game-file inputs in CI. */
#include <sys/stat.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdint.h>
int main(int argc,char **argv) {
    struct stat value;
    if(argc!=2 || stat(argv[1],&value)!=0) return 1;
    printf("{\"size\":%" PRId64 ",\"birth_ns\":%" PRId64 ",\"access_ns\":%" PRId64 ",\"write_ns\":%" PRId64 "}\n",
        (int64_t)value.st_size,
        (int64_t)value.st_birthtimespec.tv_sec*1000000000+value.st_birthtimespec.tv_nsec,
        (int64_t)value.st_atimespec.tv_sec*1000000000+value.st_atimespec.tv_nsec,
        (int64_t)value.st_mtimespec.tv_sec*1000000000+value.st_mtimespec.tv_nsec);
    return 0;
}
