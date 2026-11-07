// Copyright (c) Alkyhol contributors. MIT licensed.
// Overlapping register numbers in different namespaces/spaces are deliberate.
cbuffer Settings : register(b0) {
    uint count;
    uint salt;
    float2 scale;
    row_major float3x4 transform;
};
StructuredBuffer<uint> inputA : register(t0);
StructuredBuffer<uint> inputB : register(t0, space2);
RWStructuredBuffer<uint4> output : register(u0);
groupshared uint tile[8];

[numthreads(8,1,1)]
void CSMain(uint3 id : SV_DispatchThreadID, uint lane : SV_GroupIndex) {
    // All threads reach the barrier, including the padded final group.
    tile[lane] = id.x < count ? inputA[id.x] : 0;
    GroupMemoryBarrierWithGroupSync();
    if (id.x < count) {
        uint a = tile[7-lane];
        uint b = inputB[id.x];
        float3 p = mul(transform, float4(float2(id.x, 2) * scale, 3, 1));
        output[id.x] = uint4(a + salt, (b << (id.x & 7)) ^ salt,
            asuint(p.x + p.y + p.z), (id.x & 1) ? b : a);
    }
}
