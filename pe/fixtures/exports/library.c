/* Owned MIT-licensed DLL observer fixture. Built once to PE64 with open LLVM. */
typedef unsigned int U32;
U32 Data = 0x31415926u;
U32 Bss;
static U32 *volatile DataPointer = &Data; /* Retain an actual DIR64 fixup. */
int __stdcall DllMain(void *module, U32 reason, void *reserved) {
    (void)module; (void)reserved;
    if (reason == 1) Bss = 0x13579bdfu;
    if (reason == 0) Bss = 0;
    return 1;
}
U32 __stdcall Probe(U32 a, U32 b) { return (a * 33u) ^ b ^ *DataPointer ^ Bss; }
U32 __stdcall OrdinalProbe(U32 a) { return a + 0x12345678u; }
