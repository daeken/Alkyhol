#include <Windows.h>
#include <cstddef>
static_assert(sizeof(FILETIME)==8 && sizeof(SYSTEMTIME)==16);
static_assert(alignof(SYSTEMTIME)==2);
static_assert(offsetof(SYSTEMTIME,wDayOfWeek)==4);
static_assert(offsetof(SYSTEMTIME,wMilliseconds)==14);
