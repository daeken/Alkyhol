// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include "Windows.h"
typedef void* HKEY;
typedef LONG LSTATUS;
#define HKEY_LOCAL_MACHINE ((HKEY)(intptr_t)(int32_t)0x80000002u)
#define KEY_READ 0x20019u
extern "C" LSTATUS RegOpenKeyExW(HKEY,LPCWSTR,DWORD,DWORD,HKEY*);
extern "C" LSTATUS RegQueryValueExW(HKEY,LPCWSTR,DWORD*,DWORD*,BYTE*,DWORD*);
extern "C" LSTATUS RegCloseKey(HKEY);
#define RegOpenKeyEx RegOpenKeyExW
#define RegQueryValueEx RegQueryValueExW
