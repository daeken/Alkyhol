#include "ctx-observe.h"
/* No embedded manifest: importing DLL's binding must select this private file. */
int __stdcall DllMain(H module,U32 reason,H reserved) {
    (void)reserved;if(reason==0||reason==1)observe(1,reason,module);return 1;
}
__declspec(dllexport) U32 __stdcall ProviderValue(void) {return 0x13572468;}
