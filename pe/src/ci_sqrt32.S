/* Original MIT bounded x87 gateway. These are guest i386 instructions, not a
 * host floating-point call. Failure is patched to the checked diagnostic gate. */
.text
.code32
.p2align 4
.globl _alkyhol_ci_sqrt32_start
.globl _alkyhol_ci_sqrt32_end
.globl _alkyhol_ci_sqrt32_failure
_alkyhol_ci_sqrt32_start:
    pushfl
    pushal
    movl %esp,%ebp
    subl $544,%esp
    andl $-16,%esp
    fxsave (%esp)
    /* Only two measured, fully masked round-to-nearest profiles are admitted. */
    cmpw $0x027f,(%esp)
    je 0f
    cmpw $0x037f,(%esp)
    jne 9f
0:
    movl 24(%esp),%eax
    andl $0xffffffc0,%eax
    cmpl $0x1f80,%eax
    jne 9f
    /* FTW has physical-register bits; TOP identifies logical ST(0). */
    movzwl 2(%esp),%ecx
    shrl $11,%ecx
    andl $7,%ecx
    movzbl 4(%esp),%eax
    btl %ecx,%eax
    jnc 9f
    movzwl 40(%esp),%eax
    movl 32(%esp),%ecx
    movl 36(%esp),%edx
    /* Either sign of exact zero is valid. Negative finite/NaN are not. */
    testl $0x7fff,%eax
    jnz 1f
    orl %edx,%ecx
    jnz 9f
    jmp 8f
1:
    cmpl $0x3bcd,%eax /* minimum positive binary64 subnormal */
    jb 9f
    cmpl $0x43fe,%eax
    ja 9f
    testl $0x80000000,%edx /* canonical extended significand */
    jz 9f
    cmpl $0x43fe,%eax
    jne 8f
    cmpl $0xffffffff,%edx
    jne 8f
    cmpl $0xfffff800,%ecx /* maximum finite binary64 operand */
    ja 9f
8:
    fsqrt
    cmpw $0x037f,(%esp)
    jne 7f
    fstpl 512(%esp)
    fldl 512(%esp)
7:
    movl %ebp,%esp
    popal
    popfl
    retl
9:
    /* Keep the real snapshot in the checked guest stack until the diagnostic
     * gate terminates. The second argument identifies the original caller. */
    movl %esp,%ecx
    pushl 36(%ebp)
    pushl %ecx
    .byte 0xb8 /* MOV imm32,EAX; CALL transfers to the private rejection gate. */
_alkyhol_ci_sqrt32_failure:
    .long 0
    calll *%eax
    ud2
_alkyhol_ci_sqrt32_end:
.code64
