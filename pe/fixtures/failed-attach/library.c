/* Owned MIT fixture: genuine imported APIs and deliberately failed DLL attach. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL void * __stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *, const void *, U32, U32 *, void *);
DLL void __stdcall SetLastError(U32);
static U32 Counter;
static U32 *volatile CounterPointer = &Counter; /* A real DIR64 relocation. */
int __stdcall DllMain(void *module, U32 reason, void *reserved) {
    (void)module;
    if (reason == 1 || reason == 0) {
        U32 record[4], written = 0;
        record[0] = reason;
        record[1] = ++*CounterPointer;
        record[2] = reserved == (void *)0;
        record[3] = 0xa17ac0deu;
        WriteFile(GetStdHandle(0xfffffff5u), record, sizeof(record), &written, (void *)0);
        SetLastError(0x1234);
    }
    return reason != 1;
}
