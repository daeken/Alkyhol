#include <windows.h>
#include <cstddef>
static_assert(sizeof(void*)==4 && sizeof(SYSTEM_INFO)==36);
static_assert(offsetof(SYSTEM_INFO,dwPageSize)==4);
static_assert(offsetof(SYSTEM_INFO,lpMinimumApplicationAddress)==8);
static_assert(offsetof(SYSTEM_INFO,lpMaximumApplicationAddress)==12);
static_assert(offsetof(SYSTEM_INFO,dwActiveProcessorMask)==16);
static_assert(offsetof(SYSTEM_INFO,dwNumberOfProcessors)==20);
static_assert(offsetof(SYSTEM_INFO,dwProcessorType)==24);
static_assert(offsetof(SYSTEM_INFO,dwAllocationGranularity)==28);
static_assert(offsetof(SYSTEM_INFO,wProcessorLevel)==32);
static_assert(offsetof(SYSTEM_INFO,wProcessorRevision)==34);
