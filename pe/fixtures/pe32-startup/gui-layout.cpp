// Original MIT independent Windows SDK ABI assertions for the PE32 GUI adapter.
#include <Windows.h>
#include <cstddef>
static_assert(sizeof(void*)==4);
static_assert(sizeof(MSG)==28);
static_assert(offsetof(MSG,pt)==20);
static_assert(sizeof(PAINTSTRUCT)==64);
static_assert(offsetof(PAINTSTRUCT,rcPaint)==8);
static_assert(sizeof(BITMAP)==24);
static_assert(offsetof(BITMAP,bmBits)==20);
static_assert(sizeof(DIBSECTION)==84);
static_assert(offsetof(DIBSECTION,dsBmih)==24);
static_assert(offsetof(DIBSECTION,dshSection)==76);
static_assert(sizeof(WNDCLASSEXW)==48);
static_assert(sizeof(CREATESTRUCTW)==48);

static_assert(sizeof(OVERLAPPED)==20);
static_assert(offsetof(OVERLAPPED,hEvent)==16);
static_assert(offsetof(FILE_NOTIFY_INFORMATION,FileName)==12);

static_assert(sizeof(LOGBRUSH)==12);
