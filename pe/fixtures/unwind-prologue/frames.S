/* Owned MIT frame. Real linker metadata; labels mark unwind operation ends. */
.intel_syntax noprefix
.text
.p2align 4
.globl target_begin,after_rbx,after_r12,after_alloc,after_save,target_body,target_end
.seh_proc target_begin
target_begin:
    push rbx
    .seh_pushreg rbx
after_rbx:
    push r12
    .seh_pushreg r12
after_r12:
    sub rsp,168
    .seh_stackalloc 168
after_alloc:
    mov [rsp+40],r13
    .seh_savereg r13,40
after_save:
    movdqa [rsp+64],xmm6
    .seh_savexmm xmm6,64
    .seh_endprologue
target_body:
    nop
    movabs rbx,0xaaaaaaaa33333333
    movabs r12,0xcccccccc55555555
    movabs r13,0xdddddddd66666666
    pcmpeqd xmm6,xmm6
    mov [rcx+144],rbx
    mov [rcx+152],rsp
    mov [rcx+216],r12
    mov [rcx+224],r13
    movdqu [rcx+512],xmm6
    lea rax,[rip+target_body]
    mov [rcx+248],rax
    call inspect_context
    movdqa xmm6,[rsp+64]
    mov r13,[rsp+40]
    add rsp,168
    pop r12
    pop rbx
    ret
target_end:
.seh_endproc
