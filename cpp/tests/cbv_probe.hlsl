// Copyright (c) Alkyhol contributors. MIT licensed.
// Native reference consumer for the finite CBV/copy descriptor workload.
cbuffer A : register(b0) { uint4 a; };
cbuffer B : register(b1) { uint4 b; };
cbuffer C : register(b2) { uint4 c; };
cbuffer D : register(b3) { uint4 d; };
RWStructuredBuffer<uint> output : register(u0);
[RootSignature("DescriptorTable(CBV(b0, numDescriptors=4)), UAV(u0)")]
[numthreads(1,1,1)]
void main() {
    for(uint i=0;i<4;++i){output[i]=a[i];output[4+i]=b[i];output[8+i]=c[i];output[12+i]=d[i];}
}
