#include <Windows.h>
#include <cstddef>
static_assert(sizeof(FILETIME)==8);
static_assert(sizeof(TIME_ZONE_INFORMATION)==172);
static_assert(offsetof(TIME_ZONE_INFORMATION,StandardBias)==84);
static_assert(offsetof(TIME_ZONE_INFORMATION,DaylightBias)==168);
