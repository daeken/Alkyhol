/* Original MIT independent SDK boot-clock reader for host assertions. */
#include <time.h>
#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
/* SDK-typed independent clock API. Apple Libc documents RAW as the continuous
 * Mach boot timebase; CLOCK_MONOTONIC instead subtracts wall-clock boot time.
 * https://github.com/apple-oss-distributions/Libc/blob/main/gen/clock_gettime.c */
int main(void){struct timespec value;if(clock_gettime(CLOCK_MONOTONIC_RAW,&value))return 1;printf("%" PRIu64 "\n",(uint64_t)value.tv_sec*1000+(uint64_t)value.tv_nsec/1000000);return 0;}
