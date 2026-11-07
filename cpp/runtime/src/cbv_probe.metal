// Copyright (c) Alkyhol contributors. MIT licensed.
// Diagnostic GPU consumer of virtual CBV records, not a general D3D dispatcher.
#include <metal_stdlib>
using namespace metal;
struct Record { ulong tag; ulong address; ulong length; ulong reserved[5]; };
kernel void read_cbvs(device const Record* table [[buffer(0)]], device uint* output [[buffer(1)]], uint index [[thread_position_in_grid]]) {
    if(index>=4)return;
    Record record=table[index];if(record.tag!=1||record.length<16)return;
    device const uint* source=reinterpret_cast<device const uint*>(record.address);
    for(uint i=0;i<4;++i)output[index*4+i]=source[i];
}
