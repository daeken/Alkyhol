/* Original MIT software-exception entry. Preserve guest integer/FP state. */
.text
.code32
.globl _alkyhol_raise32_start,_alkyhol_raise32_end,_alkyhol_raise32_service
_alkyhol_raise32_start:
    pushfl
    pushal
    movl %esp,%ebp
    subl $528,%esp
    andl $-16,%esp
    movl %esp,%esi
    fxsave (%esi)
    pushl $0
    pushl $0x53454832
    pushl %esi
    pushl %ebp
    .byte 0xb8
_alkyhol_raise32_service:
    .long 0
    calll *%eax
    fxrstor (%esi)
    movl %ebp,%esp
    popal
    popfl
    retl $16
_alkyhol_raise32_end:
.code64
