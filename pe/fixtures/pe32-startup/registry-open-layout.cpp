#include <windows.h>
static_assert(sizeof(void*)==4 && sizeof(HKEY)==4);
static_assert(sizeof(REGSAM)==4 && sizeof(LSTATUS)==4);
static_assert(KEY_READ==0x20019 && KEY_WOW64_32KEY==0x200 && KEY_WOW64_64KEY==0x100);
