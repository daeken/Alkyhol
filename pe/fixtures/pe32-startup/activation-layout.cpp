// Independent Windows SDK ABI checks for the freestanding i386 observer.
#include <windows.h>
#include <cstddef>
static_assert(sizeof(void *) == 4);
static_assert(sizeof(ACTCTXW) == 32);
static_assert(offsetof(ACTCTXW, lpSource) == 8);
static_assert(offsetof(ACTCTXW, lpResourceName) == 20);
static_assert(offsetof(ACTCTXW, hModule) == 28);
static_assert(sizeof(ACTCTX_SECTION_KEYED_DATA) == 64);
static_assert(offsetof(ACTCTX_SECTION_KEYED_DATA, hActCtx) == 32);
static_assert(offsetof(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex) == 36);
static_assert(offsetof(ACTCTX_SECTION_KEYED_DATA, ulFlags) == 40);
static_assert(offsetof(ACTCTX_SECTION_KEYED_DATA, AssemblyMetadata) == 44);
