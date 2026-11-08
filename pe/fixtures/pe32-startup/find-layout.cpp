#include <Windows.h>
#include <cstddef>
static_assert(sizeof(WIN32_FIND_DATAW)==592);
static_assert(offsetof(WIN32_FIND_DATAW,ftCreationTime)==4);
static_assert(offsetof(WIN32_FIND_DATAW,nFileSizeHigh)==28);
static_assert(offsetof(WIN32_FIND_DATAW,cFileName)==44);
static_assert(offsetof(WIN32_FIND_DATAW,cAlternateFileName)==564);
