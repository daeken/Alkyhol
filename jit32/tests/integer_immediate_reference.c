// Owned independent x86 observer: declared-width immediate arithmetic and flags.
// Runs as real x86 assembly under Rosetta; it does not use SharpRetro semantics.
#include <stdint.h>
#include <stdio.h>
static void row(uint32_t width, uint32_t op, uint32_t imm,
                uint32_t input, uint32_t result, uint64_t flags, uint32_t carry) {
    uint32_t data[] = {width, op, imm, input, result, (uint32_t)flags, carry};
    if (fwrite(data, sizeof(data), 1, stdout) != 1) __builtin_trap();
}
#define ONE(SIZE, REG, W, OP, ID, IMM) do { \
    uint32_t result = input; uint64_t flags; \
    __asm__ volatile("btl $0, %[carry]\n\t" OP SIZE " %[imm], %" REG "[value]\n\tpushfq\n\tpopq %[flags]" \
        : [value] "+r"(result), [flags] "=r"(flags) \
        : [carry] "r"(carry), [imm] "i"(IMM) : "cc"); \
    row(W, ID, (uint32_t)(IMM), input, result, flags, carry); \
} while (0)
#define FAMILY(SIZE,REG,W,IMM) do { \
    ONE(SIZE,REG,W,"cmp",0,IMM); ONE(SIZE,REG,W,"sub",1,IMM); \
    ONE(SIZE,REG,W,"add",2,IMM); ONE(SIZE,REG,W,"adc",3,IMM); ONE(SIZE,REG,W,"sbb",4,IMM); \
} while (0)
int main(void) {
    const uint32_t values[] = {0,1,0x7f,0x80,0x7fff,0x8000,0xfffe0200,0xffffffff};
    for (unsigned i = 0; i < sizeof(values)/sizeof(values[0]); ++i) {
        uint32_t input = values[i];
        for (uint32_t carry=0; carry<2; ++carry) {
            FAMILY("b","b",8,-1); FAMILY("b","b",8,-128); FAMILY("b","b",8,127);
            FAMILY("w","w",16,-1); FAMILY("w","w",16,-32768); FAMILY("w","w",16,127);
            FAMILY("l","k",32,-1); FAMILY("l","k",32,(-2147483647-1));
            FAMILY("l","k",32,127); FAMILY("l","k",32,-128);
            FAMILY("l","k",32,(int32_t)0xfffe0200); FAMILY("l","k",32,(int32_t)0xffff0200);
        }
    }
    return 0;
}
