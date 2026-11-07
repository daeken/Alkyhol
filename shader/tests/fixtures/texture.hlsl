// Copyright (c) Alkyhol contributors. MIT licensed.
Texture2D<float4> image : register(t0);
SamplerState pointSampler : register(s0);
RWStructuredBuffer<float4> output : register(u0);
[numthreads(2,2,1)]
void CSMain(uint3 id : SV_DispatchThreadID) {
    output[id.y * 2 + id.x] = image.SampleLevel(pointSampler,(float2(id.xy)+0.5)/2.0,0);
}
