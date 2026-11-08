/* Original MIT PE32 register canaries and actual FS-linked exception frames. */
.text
.def @feat.00; .scl 3; .type 0; .endef
.globl @feat.00
.set @feat.00,1
.globl _fault32,_fault_site,_resume_site
_fault32:
    pushl %ebp
    pushl %ebx
    pushl %esi
    pushl %edi
    pushl $_continue_handler
    pushl %fs:0
    movl %esp,%fs:0
    movl %esp,_outer_frame
#if NEGATIVE==4
    pushl $_unregistered_handler
#else
    pushl $_search_handler
#endif
    pushl %fs:0
    movl %esp,%fs:0
    movl %esp,_inner_frame
    movl $0x12345678,%eax
    movl $0x23456789,%ebx
    movl $0x3456789a,%ecx
    movl $0x456789ab,%edx
    movl $0x56789abc,%esi
    movl $0x6789abcd,%edi
    movl $0x789abcde,%ebp
    clc
_fault_site:
    ud2
_resume_site:
    movl %eax,_after_eax
    movl %ebx,_after_ebx
    pushfl
    popl %eax
    movl %eax,_after_flags
    movl (%esp),%eax
    movl %eax,%fs:0
    addl $8,%esp
    movl (%esp),%eax
    movl %eax,%fs:0
    addl $8,%esp
    popl %edi
    popl %esi
    popl %ebx
    popl %ebp
    cld
    retl
.safeseh _search_handler
.safeseh _continue_handler
