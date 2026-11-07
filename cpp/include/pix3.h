// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#if defined(USE_PIX)
#error PIX event recording is not implemented; this header supports disabled instrumentation only.
#endif
// The upstream release configuration does not enable PIX instrumentation.
template<class... T> inline void PIXBeginEvent(T&&...){}
template<class... T> inline void PIXEndEvent(T&&...){}
template<class... T> inline void PIXSetMarker(T&&...){}
