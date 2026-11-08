/* Original MIT, actual i386 x87-stack invocation, never a C double surrogate. */
.text
.globl _run_sqrt
_run_sqrt:
    pushl %ebp
    movl %esp,%ebp
    pushl %ebx
    pushl %esi
    pushl %edi
    subl $12,%esp
    movl 8(%ebp),%esi
    movl 16(%ebp),%edi
    fninit
    fldcw 12(%ebp)
    ldmxcsr mxcsr_nearest
    fldt below_value
    fldt (%esi)
    fnstcw 8(%edi)
    fnstsw 10(%edi)
    stmxcsr 40(%edi)
    calll *__imp___CIsqrt
    fnstcw 12(%edi)
    fnstsw 14(%edi)
    stmxcsr 44(%edi)
    fstpt 20(%edi)
    fstpt 30(%edi)
    fnstsw 16(%edi)
    fninit
    ldmxcsr mxcsr_nearest
    addl $12,%esp
    popl %edi
    popl %esi
    popl %ebx
    popl %ebp
    retl
.section .rdata,"dr"
.p2align 4
below_value:
    .long 0,0xc0000000
    .short 0x3fff
.p2align 2
mxcsr_nearest:
    .long 0x1f80
