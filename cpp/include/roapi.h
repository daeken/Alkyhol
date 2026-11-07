// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include "Windows.h"
enum RO_INIT_TYPE {RO_INIT_SINGLETHREADED=0,RO_INIT_MULTITHREADED=1};
extern "C" HRESULT RoInitialize(RO_INIT_TYPE);
extern "C" void RoUninitialize();
