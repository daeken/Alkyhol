/* Original MIT fixed-width observer types; SDK assertions are separate. */
typedef unsigned int U32;
typedef struct {
    U32 code,flags,nested,address,count,information[15];
} Record32;
typedef struct {
    U32 flags,debug[6],floating[28];
    U32 gs,fs,es,ds,edi,esi,ebx,edx,ecx,eax,ebp,eip,cs,eflags,esp,ss;
    U32 extended[128];
} Context32;
