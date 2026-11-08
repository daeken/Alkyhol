/* Original MIT bounded arctangent gate. Real guest x87 operands/results. */
.text
.code32
.p2align 4
.globl _alkyhol_ci_atan32_start
.globl _alkyhol_ci_atan32_end
.globl _alkyhol_ci_atan32_failure
_alkyhol_ci_atan32_start:
    pushfl
    pushal
    movl %esp,%ebp
    subl $544,%esp
    andl $-16,%esp
    fxsave (%esp)
    /* The 027f candidate disagrees with native C1; do not accept that mode. */
    cmpw $0x037f,(%esp)
    jne 9f
    movl 24(%esp),%eax
    andl $0xffffffc0,%eax
    cmpl $0x1f80,%eax
    jne 9f
    movzwl 2(%esp),%ecx
    shrl $11,%ecx
    andl $7,%ecx
    movzbl 4(%esp),%eax
    btl %ecx,%eax
    jnc 9f
    /* FLD1 needs one free physical register above the current top. */
    decl %ecx
    andl $7,%ecx
    btl %ecx,%eax
    jc 9f
    movzwl 40(%esp),%eax
    andl $0x7fff,%eax
    movl 32(%esp),%ecx
    movl 36(%esp),%edx
    testl %eax,%eax
    jnz 1f
    orl %edx,%ecx
    jnz 9f
    jmp 8f
1:
    cmpl $0x3bcd,%eax
    jb 9f
    cmpl $0x43fe,%eax
    ja 9f
    testl $0x80000000,%edx
    jz 9f
    cmpl $0x43fe,%eax
    jne 8f
    cmpl $0xffffffff,%edx
    jne 8f
    cmpl $0xfffff800,%ecx
    ja 9f
8:
    fld1
    fpatan
    fstpl 512(%esp)
    fldl 512(%esp)
    /* A nonzero finite binary input has an inexact arctangent. The observed
     * host primitive omits PE for tiny operands; preserve native sticky PE
     * without guessing C1 or changing the remaining environment. */
    movl 32(%esp),%eax
    orl 36(%esp),%eax
    jz 7f
    fnstenv 512(%esp)
    orw $0x20,516(%esp)
    fldenv 512(%esp)
7:
    movl %ebp,%esp
    popal
    popfl
    retl
9:
    movl %esp,%ecx
    pushl 36(%ebp)
    pushl %ecx
    .byte 0xb8
_alkyhol_ci_atan32_failure:
    .long 0
    calll *%eax
    ud2
_alkyhol_ci_atan32_end:
.code64
