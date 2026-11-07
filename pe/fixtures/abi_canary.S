/* Explicit Win64 nonvolatile GPR/XMM canaries across two real imported calls.
 * This function itself preserves the caller's originals, including XMM6..15. */
.intel_syntax noprefix
.text
.globl abi_canary
abi_canary:
    push rbx
    push rbp
    push rdi
    push rsi
    push r12
    push r13
    push r14
    push r15
    sub rsp, 200
    movdqu [rsp + 32], xmm6
    movdqu [rsp + 48], xmm7
    movdqu [rsp + 64], xmm8
    movdqu [rsp + 80], xmm9
    movdqu [rsp + 96], xmm10
    movdqu [rsp + 112], xmm11
    movdqu [rsp + 128], xmm12
    movdqu [rsp + 144], xmm13
    movdqu [rsp + 160], xmm14
    movdqu [rsp + 176], xmm15
    mov rbx, 305397760
    mov rbp, 305397761
    mov rdi, 305397762
    mov rsi, 305397763
    mov r12, 305397764
    mov r13, 305397765
    mov r14, 305397766
    mov r15, 305397767
    movdqu xmm6, [rip + canary]
    movdqu xmm7, [rip + canary]
    movdqu xmm8, [rip + canary]
    movdqu xmm9, [rip + canary]
    movdqu xmm10, [rip + canary]
    movdqu xmm11, [rip + canary]
    movdqu xmm12, [rip + canary]
    movdqu xmm13, [rip + canary]
    movdqu xmm14, [rip + canary]
    movdqu xmm15, [rip + canary]
    mov ecx, 0x11223344
    call qword ptr [rip + __imp_SetLastError]
    call qword ptr [rip + __imp_GetLastError]
    cmp eax, 0x11223344
    jne failed
    cmp rbx, 305397760
    jne failed
    cmp rbp, 305397761
    jne failed
    cmp rdi, 305397762
    jne failed
    cmp rsi, 305397763
    jne failed
    cmp r12, 305397764
    jne failed
    cmp r13, 305397765
    jne failed
    cmp r14, 305397766
    jne failed
    cmp r15, 305397767
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm6
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm7
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm8
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm9
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm10
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm11
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm12
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm13
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm14
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    movdqu xmm0, [rip + canary]
    pcmpeqd xmm0, xmm15
    pmovmskb eax, xmm0
    cmp eax, 65535
    jne failed
    xor eax, eax
    jmp restore
failed:
    mov eax, 1
restore:
    movdqu xmm6, [rsp + 32]
    movdqu xmm7, [rsp + 48]
    movdqu xmm8, [rsp + 64]
    movdqu xmm9, [rsp + 80]
    movdqu xmm10, [rsp + 96]
    movdqu xmm11, [rsp + 112]
    movdqu xmm12, [rsp + 128]
    movdqu xmm13, [rsp + 144]
    movdqu xmm14, [rsp + 160]
    movdqu xmm15, [rsp + 176]
    add rsp, 200
    pop r15
    pop r14
    pop r13
    pop r12
    pop rsi
    pop rdi
    pop rbp
    pop rbx
    ret
.section .rdata,"dr"
.p2align 4
canary:
    .quad 0x0123456789abcdef, 0xfedcba9876543210
