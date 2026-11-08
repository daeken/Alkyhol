/* Original MIT CPU gates. x86 descriptor API/layout references: docs/pe32.md.
 * No Wine implementation code is copied. All executable templates are copied
 * into owned low memory and patched while RW, then protected RX. */
.text
.globl _alkyhol_enter_pe32
_alkyhol_enter_pe32:
    pushq %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    xorl %eax,%eax
    movw %ds,%ax
    pushq %rax
    movw %es,%ax
    pushq %rax
    movw %ss,%ax
    pushq %rax
    movq %rsp,%r12
    leaq 1f(%rip),%r13
    movl %edx,%r14d
    movw %cx,%ds
    movw %cx,%es
    movw %cx,%ss
    movq %rsi,%rsp
    pushq %rdx
    pushq %rdi
    lretq
1:
    movl %eax,%r11d
    popq %rax
    movw %ax,%ss
    popq %rax
    movw %ax,%es
    popq %rax
    movw %ax,%ds
    movl %r11d,%eax
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    popq %rbp
    retq

.p2align 4
.globl _alkyhol_pe32_gate_start,_alkyhol_pe32_gate_end,_alkyhol_pe32_dispatch_slot
_alkyhol_pe32_gate_start:
    movq %rsp,%r15
    movq %r12,%rsp
    pushq %rbx
    pushq %rbp
    pushq %rsi
    pushq %rdi
    movl (%r15),%edi
    leaq 12(%r15),%rsi
    cld
    callq *_alkyhol_pe32_dispatch_slot(%rip)
    popq %rdi
    popq %rsi
    popq %rbp
    popq %rbx
    movl 4(%r15),%ecx
    leaq 8(%r15),%rsp
    pushq %r14
    pushq %rcx
    lretq
.p2align 3
_alkyhol_pe32_dispatch_slot:
    .quad 0
_alkyhol_pe32_gate_end:
