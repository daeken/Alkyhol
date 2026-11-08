/* Installed SDK assertions for the independent Rust signal ABI declarations. */
#include <signal.h>
#include <sys/ucontext.h>
#include <stddef.h>
_Static_assert(sizeof(struct sigaction)==16,"action size");
_Static_assert(offsetof(struct sigaction,sa_mask)==8,"signal mask offset");
_Static_assert(offsetof(struct sigaction,sa_flags)==12,"signal flags offset");
_Static_assert(sizeof(stack_t)==24,"alternate stack size");
_Static_assert(offsetof(ucontext_t,uc_mcsize)==40,"context size field");
_Static_assert(offsetof(ucontext_t,uc_mcontext)==48,"context pointer field");
_Static_assert(offsetof(struct __darwin_mcontext64,__ss)==16,"register prefix");
_Static_assert(sizeof(struct __darwin_x86_thread_state64)==168,"register width");
_Static_assert(offsetof(struct __darwin_x86_thread_state64,__rip)==128,"PC offset");
_Static_assert(offsetof(struct __darwin_x86_thread_state64,__fs)==152,"FS offset");
_Static_assert(offsetof(siginfo_t,si_code)==8,"signal code offset");
_Static_assert(SIGILL==4 && (SA_ONSTACK|SA_SIGINFO|SA_NODEFER)==0x51,"signal constants");
