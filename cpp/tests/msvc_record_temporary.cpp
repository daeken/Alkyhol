// Copyright (c) Alkyhol contributors. MIT licensed.
// The C-style reference cast is deliberately the MSVC construct used upstream.
#include <cassert>
#include <cstdint>
#include <cstdio>
static int destroyed=0;
struct alignas(16) Source { std::uint32_t words[4]; ~Source(){++destroyed;} };
struct alignas(16) Target {std::uint32_t words[4];};
static Source make(){return Source{{0x12345678,0xffffffff,0x80000000,0xabcdef01}};}
static Target copy(){return (Target&)(make());}
int main(){auto result=copy();assert(destroyed>=1);assert(result.words[0]==0x12345678);assert(result.words[1]==0xffffffff);assert(result.words[2]==0x80000000);assert(result.words[3]==0xabcdef01);std::puts("MSVC record temporary: storage bits and completed lifetime passed");}
