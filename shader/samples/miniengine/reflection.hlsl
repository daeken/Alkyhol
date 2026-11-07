// Copyright (c) Alkyhol contributors. MIT licensed.
// Queue adapter for unchanged original SSR ray generation. The original entry
// emits one ray and does not consume its payload; hit shading runs next.
static uint3 alkyholDispatch;
#define DispatchRaysIndex() alkyholDispatch
#define HLSL
#include "original/ModelViewerRaytracing.h"
#include "context.hlsli"
StructuredBuffer<AlkyholHitRecord> alkyholPrimary : register(t21);
RWStructuredBuffer<AlkyholHitRecord> alkyholSecondary : register(u21);
RWStructuredBuffer<uint> alkyholCounts : register(u20);
void AlkyholReflectionTrace(RaytracingAccelerationStructure acceleration,uint flags,uint mask,uint contribution,uint multiplier,uint missIndex,RayDesc ray,inout RayPayload payload)
{
    uint index=alkyholDispatch.y*alkyholWidth+alkyholDispatch.x;
    AlkyholHitRecord record=(AlkyholHitRecord)0;record.mesh=0xffffffff;
    record.origin=float4(ray.Origin,ray.TMin);record.direction=float4(ray.Direction,ray.TMax);
    RayQuery<RAY_FLAG_FORCE_OPAQUE> query;query.TraceRayInline(acceleration,flags,mask,ray);while(query.Proceed()) {}
    InterlockedAdd(alkyholCounts[37],1);
    if(query.CommittedStatus()==COMMITTED_TRIANGLE_HIT) {
        InterlockedAdd(alkyholCounts[38],1);record.mesh=query.CommittedInstanceIndex();record.primitive=query.CommittedPrimitiveIndex();record.bary=query.CommittedTriangleBarycentrics();record.distance=query.CommittedRayT();
        if(record.mesh==alkyholPrimary[index].mesh&&record.distance<0.2)InterlockedAdd(alkyholCounts[40],1);
    }
    alkyholSecondary[index]=record;
}
#define TraceRay AlkyholReflectionTrace
#include "original/RayGenerationShaderSSRLib.hlsl"
[numthreads(8,8,1)]
void AlkyholMain(uint3 id:SV_DispatchThreadID)
{
    if(id.x>=alkyholWidth||id.y>=alkyholHeight)return;
    AlkyholHitRecord empty=(AlkyholHitRecord)0;empty.mesh=0xffffffff;alkyholSecondary[id.y*alkyholWidth+id.x]=empty;
    alkyholDispatch=id;RayGen();
}
