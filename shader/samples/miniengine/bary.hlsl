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
#include "context.hlsli"
RWStructuredBuffer<AlkyholHitRecord> alkyholHits : register(u21);
RWTexture2D<float> alkyholDepth : register(u22);
RWTexture2D<float4> alkyholNormals : register(u23);

RWStructuredBuffer<uint> alkyholCounts : register(u20);

void alkyholTraceRay(RaytracingAccelerationStructure acceleration, uint flags,
    uint mask, uint contribution, uint geometryMultiplier, uint missIndex,
    RayDesc ray, inout RayPayload payload)
{
    RayQuery<RAY_FLAG_FORCE_OPAQUE> query;
    query.TraceRayInline(acceleration, flags, mask, ray);
    while(query.Proceed()) {}
    AlkyholHitRecord record=(AlkyholHitRecord)0;
    record.mesh=0xffffffff;record.origin=float4(ray.Origin,ray.TMin);record.direction=float4(ray.Direction,ray.TMax);
    if(query.CommittedStatus() == COMMITTED_TRIANGLE_HIT) {
        alkyholHitT=query.CommittedRayT();
        BuiltInTriangleIntersectionAttributes attributes;
        attributes.barycentrics=query.CommittedTriangleBarycentrics();
        record.bary=attributes.barycentrics;record.distance=alkyholHitT;record.mesh=query.CommittedInstanceIndex();record.primitive=query.CommittedPrimitiveIndex();
        float4 clip=mul(alkyholWorldToClip,float4(ray.Origin+ray.Direction*record.distance,1));
        alkyholDepth[alkyholDispatch.xy]=clip.z/clip.w;
        Hit(payload,attributes);
        InterlockedAdd(alkyholCounts[query.CommittedInstanceIndex()],1);
    } else {
        alkyholDepth[alkyholDispatch.xy]=0;
        Miss(payload);
        InterlockedAdd(alkyholCounts[34],1);
    }
    alkyholHits[alkyholDispatch.y*alkyholWidth+alkyholDispatch.x]=record;
}
#define TraceRay alkyholTraceRay
#include "original/RayGenerationShaderLib.hlsl"

[numthreads(8,8,1)]
void AlkyholMain(uint3 id : SV_DispatchThreadID)
{
    if(any(id.xy >= uint2(g_dynamic.resolution))) return;
    alkyholDispatch=id;
    alkyholNormals[id.xy]=float4(0,0,0,0);
    RayGen();
}
