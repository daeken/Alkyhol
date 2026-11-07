// Copyright (c) Alkyhol contributors. MIT licensed.
// Source compatibility adapter: unchanged DXR entry bodies are called from a
// compute entry; real inline queries supply their builtins. This is a fixed
// triangle/opaque, one-hit-group pipeline, not general DXR library translation.
static uint3 alkyholDispatch;
static float alkyholHitT;
#define DispatchRaysIndex() alkyholDispatch
#define RayTCurrent() alkyholHitT
#include "original/hitShaderLib.hlsl"
#include "original/missShaderLib.hlsl"

RWStructuredBuffer<uint> alkyholCounts : register(u20);

void alkyholTraceRay(RaytracingAccelerationStructure acceleration, uint flags,
    uint mask, uint contribution, uint geometryMultiplier, uint missIndex,
    RayDesc ray, inout RayPayload payload)
{
    RayQuery<RAY_FLAG_FORCE_OPAQUE> query;
    query.TraceRayInline(acceleration, flags, mask, ray);
    while(query.Proceed()) {}
    if(query.CommittedStatus() == COMMITTED_TRIANGLE_HIT) {
        alkyholHitT=query.CommittedRayT();
        BuiltInTriangleIntersectionAttributes attributes;
        attributes.barycentrics=query.CommittedTriangleBarycentrics();
        Hit(payload,attributes);
        InterlockedAdd(alkyholCounts[query.CommittedInstanceIndex()],1);
    } else {
        Miss(payload);
        InterlockedAdd(alkyholCounts[34],1);
    }
}
#define TraceRay alkyholTraceRay
#include "original/RayGenerationShaderLib.hlsl"

[numthreads(8,8,1)]
void AlkyholMain(uint3 id : SV_DispatchThreadID)
{
    if(any(id.xy >= uint2(g_dynamic.resolution))) return;
    alkyholDispatch=id;
    RayGen();
}
