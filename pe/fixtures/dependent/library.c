/* Owned MIT fixture: named code/data IAT imports from an unchanged second DLL. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL U32 __stdcall Direct(U32, U32);
DLL U32 __stdcall Forwarded(void);
DLL U32 Data;
DLL U32 Bss;
DLL void * __stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *, const void *, U32, U32 *, void *);
static U32 Counter;
static U32 *volatile CounterPointer = &Counter; /* Retain a real DIR64 fixup. */
static U32 Initial;
int __stdcall DllMain(void *module, U32 reason, void *reserved) {
    (void)module;
    if (reason == 1 || reason == 0) {
        U32 record[6], written = 0;
        if (reason == 1) Initial = Direct(3, 5);
        record[0] = reason;
        record[1] = ++*CounterPointer;
        record[2] = reserved == (void *)0;
        record[3] = Direct(3, 5);
        record[4] = Data;
        record[5] = Bss;
        WriteFile(GetStdHandle(0xfffffff5u), record, sizeof(record), &written, (void *)0);
    }
    return 1;
}
U32 __stdcall Evaluate(U32 a, U32 b) { return Direct(a,b) ^ Data ^ Initial ^ *CounterPointer; }
U32 __stdcall ReadError(void) { return Forwarded(); }
