/* Owned MIT fixture: mixed named and NONAME code/data ordinal imports. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL U32 __stdcall Direct(U32, U32);
DLL U32 __stdcall OrdinalCall(U32);
DLL U32 DataByOrdinal;
U32 Initial;
static U32 *volatile InitialPointer = &Initial; /* Actual DIR64 relocation. */
int __stdcall DllMain(void *module, U32 reason, void *reserved) {
    (void)module; (void)reserved;
    if (reason == 1) *InitialPointer = OrdinalCall(9);
    if (reason == 0) *InitialPointer = 0;
    return 1;
}
U32 __stdcall Evaluate(U32 a, U32 b) { return OrdinalCall(a) ^ Direct(a,b) ^ DataByOrdinal ^ *InitialPointer; }
