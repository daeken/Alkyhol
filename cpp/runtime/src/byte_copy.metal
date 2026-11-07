// Copyright (c) Alkyhol contributors. MIT licensed.
// Backend utility for D3D12 byte-granular buffer copies, not a sample shader.
#include <metal_stdlib>
using namespace metal;
struct CopyBytes { ulong source_offset; ulong destination_offset; ulong length; };
kernel void alkyhol_copy_bytes(device const uchar* source [[buffer(0)]],
                              device uchar* destination [[buffer(1)]],
                              constant CopyBytes& copy [[buffer(2)]],
                              uint index [[thread_position_in_grid]]) {
    if (ulong(index) < copy.length)
        destination[copy.destination_offset + ulong(index)] = source[copy.source_offset + ulong(index)];
}
