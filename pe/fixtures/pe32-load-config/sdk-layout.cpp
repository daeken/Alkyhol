// Original MIT SDK layout assertions. Compile against real Windows SDK headers.
#include <Windows.h>
#include <cstddef>
using C = IMAGE_LOAD_CONFIG_DIRECTORY32;
static_assert(offsetof(C, Size) == 0);
static_assert(offsetof(C, ProcessHeapFlags) == 44);
static_assert(offsetof(C, ProcessAffinityMask) == 48);
static_assert(offsetof(C, SecurityCookie) == 60);
static_assert(offsetof(C, SEHandlerTable) == 64);
static_assert(offsetof(C, SEHandlerCount) == 68);
static_assert(sizeof(C::SecurityCookie) == 4);
static_assert(sizeof(C::SEHandlerTable) == 4);
