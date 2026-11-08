#include <windows.h>
#include <cstddef>
static_assert(sizeof(void*)==4 && sizeof(LONG)==4);
static_assert(sizeof(POINT)==8 && offsetof(POINT,x)==0 && offsetof(POINT,y)==4);
