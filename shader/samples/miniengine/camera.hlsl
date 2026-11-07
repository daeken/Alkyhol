// Copyright (c) Alkyhol contributors. MIT licensed.
// Specialized ray-generation queue adapter. Original RayGen only emits its
// primary ray; it does not consume the returned payload. Traversal follows in
// the next Metal pass. This adapter does not claim general TraceRay semantics.
static uint3 alkyholDispatch;
#define DispatchRaysIndex() alkyholDispatch
#define HLSL
#include "original/ModelViewerRaytracing.h"
RWStructuredBuffer<float4> alkyholRays : register(u20);
void alkyholEnqueueRay(RaytracingAccelerationStructure acceleration, uint flags,
    uint mask, uint contribution, uint geometryMultiplier, uint missIndex,
    RayDesc ray, inout RayPayload payload)
{
    uint offset=(alkyholDispatch.y*uint(g_dynamic.resolution.x)+alkyholDispatch.x)*2;
    alkyholRays[offset]=float4(ray.Origin,ray.TMin);
    alkyholRays[offset+1]=float4(ray.Direction,ray.TMax);
}
#define TraceRay alkyholEnqueueRay
#include "original/RayGenerationShaderLib.hlsl"
[numthreads(8,8,1)]
void AlkyholMain(uint3 id : SV_DispatchThreadID)
{
    if(any(id.xy>=uint2(g_dynamic.resolution))) return;
    alkyholDispatch=id;
    RayGen();
}
