/* Owned MIT. The assembler emits real static metadata for both DLL functions. */
.intel_syntax noprefix
.text
.p2align 4
.globl DllMain,Probe
.seh_proc DllMain
DllMain:
    sub rsp,40
    .seh_stackalloc 40
    .seh_endprologue
    call lifecycle
    add rsp,40
    ret
.seh_endproc
.p2align 4
.seh_proc Probe
Probe:
    .seh_endprologue
    nop
    mov eax,0x42a11c0d
    ret
.seh_endproc
