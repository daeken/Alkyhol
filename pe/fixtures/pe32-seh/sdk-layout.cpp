#include <windows.h>
#include <cstddef>
#include "types.h"
static_assert(sizeof(Record32)==sizeof(EXCEPTION_RECORD32));
static_assert(sizeof(Context32)==sizeof(WOW64_CONTEXT));
static_assert(offsetof(Context32,eax)==offsetof(WOW64_CONTEXT,Eax));
static_assert(offsetof(Context32,ebx)==offsetof(WOW64_CONTEXT,Ebx));
static_assert(offsetof(Context32,eip)==offsetof(WOW64_CONTEXT,Eip));
static_assert(offsetof(Context32,esp)==offsetof(WOW64_CONTEXT,Esp));
static_assert(offsetof(Context32,eflags)==offsetof(WOW64_CONTEXT,EFlags));
static_assert(offsetof(Context32,fs)==offsetof(WOW64_CONTEXT,SegFs));
static_assert(offsetof(Context32,extended)==offsetof(WOW64_CONTEXT,ExtendedRegisters));
static_assert(offsetof(Record32,address)==offsetof(EXCEPTION_RECORD32,ExceptionAddress));
static_assert(offsetof(Record32,information)==offsetof(EXCEPTION_RECORD32,ExceptionInformation));
static_assert(WOW64_CONTEXT_CONTROL==0x10001 && WOW64_CONTEXT_INTEGER==0x10002 && WOW64_CONTEXT_SEGMENTS==0x10004);
static_assert(ExceptionContinueExecution==0 && ExceptionContinueSearch==1);
