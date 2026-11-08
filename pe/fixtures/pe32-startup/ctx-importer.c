#include "ctx-observe.h"
__declspec(dllimport) U32 __stdcall ProviderValue(void);
int __stdcall DllMain(H module,U32 reason,H reserved) {
    (void)reserved;if(reason==0||reason==1)observe(2,reason,module);return 1;
}
__declspec(dllexport) U32 __stdcall ImporterValue(void) {return ProviderValue();}
