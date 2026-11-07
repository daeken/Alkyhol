// Owned MIT native Windows SDK checks; source is not used to rebuild the PE.
#include <Windows.h>
#include <cstddef>
static_assert(sizeof(CONTEXT)==1232 && alignof(CONTEXT)==16);
static_assert(offsetof(CONTEXT,ContextFlags)==48);
static_assert(offsetof(CONTEXT,Rax)==120 && offsetof(CONTEXT,Rbx)==144);
static_assert(offsetof(CONTEXT,Rsp)==152 && offsetof(CONTEXT,Rbp)==160);
static_assert(offsetof(CONTEXT,Rsi)==168 && offsetof(CONTEXT,Rdi)==176);
static_assert(offsetof(CONTEXT,R12)==216 && offsetof(CONTEXT,Rip)==248);
static_assert(offsetof(CONTEXT,R13)==224 && offsetof(CONTEXT,R14)==232 && offsetof(CONTEXT,R15)==240);
static_assert(sizeof(RUNTIME_FUNCTION)==12 && alignof(RUNTIME_FUNCTION)==4);

static_assert(sizeof(M128A)==16);
static_assert(offsetof(CONTEXT,Xmm6)==512);
static_assert(offsetof(CONTEXT,Xmm7)==528);
static_assert(offsetof(CONTEXT,Xmm8)==544);
static_assert(offsetof(CONTEXT,Xmm9)==560);
static_assert(offsetof(CONTEXT,Xmm10)==576);
static_assert(offsetof(CONTEXT,Xmm11)==592);
static_assert(offsetof(CONTEXT,Xmm12)==608);
static_assert(offsetof(CONTEXT,Xmm13)==624);
static_assert(offsetof(CONTEXT,Xmm14)==640);
static_assert(offsetof(CONTEXT,Xmm15)==656);
