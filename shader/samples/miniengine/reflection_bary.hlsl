// Copyright (c) Alkyhol contributors. MIT licensed.
// Original barycentric closest-hit body on the queued reflected hit.
static uint3 alkyholDispatch;static float alkyholT;
#define DispatchRaysIndex() alkyholDispatch
#define RayTCurrent() alkyholT
#include "original/hitShaderLib.hlsl"
#include "context.hlsli"
StructuredBuffer<AlkyholHitRecord> alkyholHits : register(t21);
[numthreads(8,8,1)]
void AlkyholMain(uint3 id:SV_DispatchThreadID) {
    if(id.x>=alkyholWidth||id.y>=alkyholHeight)return;
    AlkyholHitRecord record=alkyholHits[alkyholWidth*alkyholHeight+id.y*alkyholWidth+id.x];
    if(record.mesh==0xffffffff)return;
    alkyholDispatch=id;alkyholT=record.distance;RayPayload payload;payload.SkipShading=false;payload.RayHitT=record.distance;
    BuiltInTriangleIntersectionAttributes attr;attr.barycentrics=record.bary;Hit(payload,attr);
}
