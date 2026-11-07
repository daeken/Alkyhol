/* Owned MIT frame. The assembler emits real .pdata/.xdata from SEH directives. */
.intel_syntax noprefix
.text
.p2align 4
.globl target_begin,target_body,target_end
.seh_proc target_begin
target_begin:
    push rbx
    .seh_pushreg rbx
    push r12
    .seh_pushreg r12
    sub rsp,40
    .seh_stackalloc 40
    .seh_endprologue
    movabs rbx,0xaaaaaaaa33333333
    movabs r12,0xcccccccc55555555
    mov [rcx+144],rbx
    mov [rcx+152],rsp
    mov [rcx+216],r12
    lea rax,[rip+target_body]
    mov [rcx+248],rax
target_body:
    nop
    call inspect_context
    add rsp,40
    pop r12
    pop rbx
    ret
target_end:
.seh_endproc
