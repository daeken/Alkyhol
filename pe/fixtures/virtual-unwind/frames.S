/* Owned MIT AMD64 frames. Capture only fields needed for manual unwinding;
 * unrelated CONTEXT bytes are seeded by C and checked for preservation. */
.intel_syntax noprefix
.text
.macro capture pc
    mov [rcx + 144], rbx
    mov [rcx + 152], rsp
    mov [rcx + 160], rbp
    mov [rcx + 168], rsi
    mov [rcx + 176], rdi
    mov [rcx + 216], r12
    mov [rcx + 224], r13
    mov [rcx + 232], r14
    mov [rcx + 240], r15
    lea rax, [rip + \pc]
    mov [rcx + 248], rax
.endm
.p2align 4
.globl small_begin, small_body, small_end
small_begin:
    push rbx
    push r12
    sub rsp, 40
    movabs rbx, 0xaaaaaaaa33333333
    movabs r12, 0xcccccccc55555555
    mov edx, 0
    capture small_body
small_body:
    nop
    call inspect_context
    add rsp, 40
    pop r12
    pop rbx
    ret
small_end:
.p2align 4
.globl large_begin, large_body, large_end
large_begin:
    push rbp
    sub rsp, 768
    movabs rbp, 0xbbbbbbbb44444444
    mov edx, 1
    capture large_body
large_body:
    nop
    call inspect_context
    add rsp, 768
    pop rbp
    ret
large_end:
.p2align 4
.globl leaf_begin, leaf_body, leaf_end
leaf_begin:
    mov edx, 2
    capture leaf_body
leaf_body:
    nop
    jmp inspect_context
leaf_end:
.section .rdata,"dr"
.p2align 2
.globl small_unwind
small_unwind:
    .byte 1,7,3,0, 7,0x42, 3,0xc0, 1,0x30, 0,0
.p2align 2
.globl large_unwind
large_unwind:
    .byte 1,8,3,0, 8,1, 96,0, 1,0x50, 0,0
.p2align 2
.globl leaf_unwind
leaf_unwind:
    .byte 1,0,0,0
