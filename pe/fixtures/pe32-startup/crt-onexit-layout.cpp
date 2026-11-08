#include <windows.h>
#include <stdlib.h>
#include <cstddef>
static_assert(sizeof(void*)==4&&sizeof(_onexit_t)==4);
static_assert(sizeof(STARTUPINFOA)==68&&sizeof(STARTUPINFOW)==68);
static_assert(offsetof(STARTUPINFOW,dwFlags)==44);
static_assert(offsetof(STARTUPINFOW,hStdInput)==56);
static_assert(offsetof(STARTUPINFOW,hStdOutput)==60);
static_assert(offsetof(STARTUPINFOW,hStdError)==64);
