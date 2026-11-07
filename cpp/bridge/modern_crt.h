// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
// The platform headers use their native Clang path. Once the original PCH is
// loaded, select its modern CRT branch instead of pre-VS2013 math fallbacks.
#include "pch.h"
#ifndef _MSC_VER
#define _MSC_VER 1930
#endif
