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
    pushq $0 /* Preserve SysV 16-byte alignment with the extra FS save. */
    movw %fs,%ax
    pushq %rax
    movq %rsp,%r12
    leaq 1f(%rip),%r13
    movl %edx,%r14d
    movw %cx,%ds
    movw %cx,%es
    movw %cx,%ss
    movw $0x1017,%ax /* Process-owned slot 514, installed for this entry. */
    movw %ax,%fs
    movq %rsi,%rsp
    pushq %rdx
    pushq %rdi
    lretq
1:
    movl %eax,%r11d
    movl %edx,%r10d /* Actual guest ESP at the near-return gate. */
    popq %rax
    movw %ax,%fs
    addq $8,%rsp
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
    movl %r10d,%edx
    shlq $32,%rdx
    orq %rdx,%rax
    retq

.p2align 4
.globl _alkyhol_pe32_gate_start,_alkyhol_pe32_gate_end,_alkyhol_pe32_dispatch_slot
_alkyhol_pe32_gate_start:
    movq %rsp,%r15
    movw (%r12),%cx
    movw %cx,%fs /* Restore host FS before calling any Rust/host code. */
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
    movw $0x1017,%cx
    movw %cx,%fs
    movl 4(%r15),%ecx
    leaq 8(%r15),%rsp
    pushq %r14
    pushq %rcx
    lretq
.p2align 3
_alkyhol_pe32_dispatch_slot:
    .quad 0
_alkyhol_pe32_gate_end:
