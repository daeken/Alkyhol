// Copyright (c) Alkyhol contributors. MIT licensed.
Texture2D<float4> images[2] : register(t0,space2);
RWStructuredBuffer<float4> output : register(u0);
[numthreads(4,1,1)]
void CSMain(uint3 id:SV_DispatchThreadID) {
    output[id.x] = images[NonUniformResourceIndex(id.x&1)].Load(int3(0,0,0));
}
