#pragma once
#include <immintrin.h>
#include <cstdint>
inline unsigned char _BitScanReverse64(unsigned long* out,uint64_t v) { if(!v)return 0;*out=63-__builtin_clzll(v);return 1; }
inline unsigned char _BitScanForward64(unsigned long* out,uint64_t v) { if(!v)return 0;*out=__builtin_ctzll(v);return 1; }
inline unsigned char _BitScanReverse(unsigned long* out,unsigned long v) { uint32_t x=v;if(!x)return 0;*out=31-__builtin_clz(x);return 1; }
inline unsigned char _BitScanForward(unsigned long* out,unsigned long v) { uint32_t x=v;if(!x)return 0;*out=__builtin_ctz(x);return 1; }
