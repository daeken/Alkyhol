/* Owned MIT real AMD64 save-to-slot frames. Only manual context restoration;
 * no exception dispatch or resumption. Probe large stack sequentially on Windows. */
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
    movdqu [rcx + 512], xmm6
    movdqu [rcx + 528], xmm7
    movdqu [rcx + 544], xmm8
    movdqu [rcx + 560], xmm9
    movdqu [rcx + 576], xmm10
    movdqu [rcx + 592], xmm11
    movdqu [rcx + 608], xmm12
    movdqu [rcx + 624], xmm13
    movdqu [rcx + 640], xmm14
    movdqu [rcx + 656], xmm15
    lea rax, [rip + \pc]
    mov [rcx + 248], rax
.endm
.macro clobber_gpr
    movabs rbx, 0xaaaaaaaa33333333
    movabs r12, 0xcccccccc55555555
.endm
.macro clobber_xmm
    movdqu xmm6, [rip + vector6]
    movdqu xmm15, [rip + vector15]
.endm
.p2align 4
.globl gpr_begin, gpr_body, gpr_end
gpr_begin:
    sub rsp, 120
gpr_alloc:
    mov [rsp + 64], rbx
gpr_save3:
    mov [rsp + 80], r12
gpr_save12:
    clobber_gpr
    mov edx, 0
    capture gpr_body
gpr_body:
    nop
    call inspect_context
    mov r12, [rsp + 80]
    mov rbx, [rsp + 64]
    add rsp, 120
    ret
gpr_end:
.p2align 4
.globl xmm_begin, xmm_body, xmm_end
xmm_begin:
    sub rsp, 120
xmm_alloc:
    movdqu [rsp + 64], xmm6
xmm_save6:
    movdqu [rsp + 96], xmm15
xmm_save15:
    clobber_xmm
    mov edx, 1
    capture xmm_body
xmm_body:
    nop
    call inspect_context
    movdqu xmm15, [rsp + 96]
    movdqu xmm6, [rsp + 64]
    add rsp, 120
    ret
xmm_end:
.p2align 4
.globl far_begin, far_body, far_end
far_begin:
    mov rax, rsp
    mov r10d, 257
far_probe:
    sub rax, 4096
    test byte ptr [rax], 0
    dec r10d
    jne far_probe
    sub rsp, 1048696
far_alloc:
    mov [rsp + 524288], rbx
far_save3:
    mov [rsp + 524304], r12
far_save12:
    movdqu [rsp + 1048576], xmm6
far_save6:
    movdqu [rsp + 1048608], xmm15
far_save15:
    clobber_gpr
    clobber_xmm
    mov edx, 2
    capture far_body
far_body:
    nop
    call inspect_context
    movdqu xmm15, [rsp + 1048608]
    movdqu xmm6, [rsp + 1048576]
    mov r12, [rsp + 524304]
    mov rbx, [rsp + 524288]
    add rsp, 1048696
    ret
far_end:
.section .rdata,"dr"
.p2align 4
vector6:
    .quad 0x1020304050607080, 0x8192a3b4c5d6e7f8
vector15:
    .quad 0xffeeddccbbaa9988, 0x7766554433221100
.p2align 2
.globl gpr_unwind
gpr_unwind:
    .byte 1, gpr_save12-gpr_begin, 5, 0
    .byte gpr_save12-gpr_begin, 0xc4
    .short 10
    .byte gpr_save3-gpr_begin, 0x34
    .short 8
    .byte gpr_alloc-gpr_begin, 0xe2
    .short 0
.p2align 2
.globl xmm_unwind
xmm_unwind:
    .byte 1, xmm_save15-xmm_begin, 5, 0
    .byte xmm_save15-xmm_begin, 0xf8
    .short 6
    .byte xmm_save6-xmm_begin, 0x68
    .short 4
    .byte xmm_alloc-xmm_begin, 0xe2
    .short 0
.p2align 2
.globl far_unwind
far_unwind:
    .byte 1, far_save15-far_begin, 15, 0
    .byte far_save15-far_begin, 0xf9
    .long 1048608
    .byte far_save6-far_begin, 0x69
    .long 1048576
    .byte far_save12-far_begin, 0xc5
    .long 524304
    .byte far_save3-far_begin, 0x35
    .long 524288
    .byte far_alloc-far_begin, 0x11
    .long 1048696
    .short 0
