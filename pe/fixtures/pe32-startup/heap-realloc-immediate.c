/* Separate owned immediate-error witness; the original 64-word producer is pinned. */
#define entry original_witness_entry
#include "heap-realloc.c"
#undef entry
U entry(void){
    H heap=GetProcessHeap();unsigned char *p=HeapAlloc(heap,0,7);if(!p)__builtin_trap();p[0]=0x71;
    SetLastError(0xface1234);H resized=HeapReAlloc(heap,0,p,0xffffffff);U error=GetLastError();
    U size=HeapSize(heap,0,p),preserved=p[0]==0x71;U released=HeapFree(heap,0,p)!=0,after=GetLastError();
    U words[10]={0x48463332,4,1,resized==0,error,size,preserved,released,after,10};
    U done=0;if(!WriteFile(GetStdHandle((U)-11),words,sizeof(words),&done,0)||done!=sizeof(words))__builtin_trap();return 0;
}
