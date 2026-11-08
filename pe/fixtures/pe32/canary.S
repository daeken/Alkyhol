/* Original MIT i386 ABI probe. Four nonvolatile integer registers and ESP
 * survive an imported stdcall call. One-byte INC establishes 32-bit decoding. */
.intel_syntax noprefix
.text
.globl _abi_canary
_abi_canary:
    push ebx
    push esi
    push edi
    push ebp
    mov ebx,0x11223344
    mov esi,0x55667788
    mov edi,0x99aabbcc
    mov ebp,esp
    push 0x13572468
    call DWORD PTR [__imp__SetLastError@4]
    call DWORD PTR [__imp__GetLastError@0]
    cmp eax,0x13572468
    jne failed
    cmp esp,ebp
    jne failed
    cmp ebx,0x11223344
    jne failed
    cmp esi,0x55667788
    jne failed
    cmp edi,0x99aabbcc
    jne failed
    mov eax,0xffffffff
    .byte 0x40
    test eax,eax
    jne failed
    mov eax,1
    jmp finished
failed:
    xor eax,eax
finished:
    mov esp,ebp
    pop ebp
    pop edi
    pop esi
    pop ebx
    ret
