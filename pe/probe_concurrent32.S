/* Rendezvous in i386 mode, then verify the thread marker at each return. */
.code32
.globl _probe_guest_start,_probe_guest_end
_probe_guest_start:
    movl 4(%esp),%edx
    movl 8(%esp),%esi
    movl 12(%esp),%ebx
    cmpl %fs:0x18,%esi
    jne 3f
    lock incl (%edx)
2:
    cmpl $2,(%edx)
    jne 2b
    movl $4096,%edi
4:
    call *%ebx
    cmpl $0x78563412,%eax
    jne 3f
    cmpl %fs:0x18,%esi
    jne 3f
    decl %edi
    jne 4b
    movl %esi,%eax
    ret $12
3:
    xorl %eax,%eax
    ret $12
_probe_guest_end:
.code64
