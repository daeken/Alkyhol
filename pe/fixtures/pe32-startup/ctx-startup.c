#include "ctx-observe.h"
__declspec(dllimport) U32 __stdcall ImporterValue(void);
U32 entry(void) {
    if(ImporterValue()!=0x13572468)__builtin_trap();
    observe(3,3,0);
    const W name[]={'c','t','x','-','i','m','p','o','r','t','e','r','.','d','l','l',0};
    H importer=GetModuleHandleW(name);if(!importer)__builtin_trap();
    struct Context input={sizeof(input),0x88,0,0,0,0,(const W *)2,0,importer};
    H context=CreateActCtxW(&input);if(context==(H)-1)__builtin_trap();
    U32 cookie=0;if(!ActivateActCtx(context,&cookie))__builtin_trap();
    observe(3,4,0);
    if(!DeactivateActCtx(0,cookie))__builtin_trap();ReleaseActCtx(context);
    observe(3,5,0);return 0;
}
