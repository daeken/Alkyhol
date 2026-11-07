// Copyright (c) Alkyhol contributors. MIT licensed.
// Fixed two-level, opaque-triangle source specialization. Material, reflection,
// camera and miss function bodies are extracted unchanged from pinned upstream.
// Full DXR library translation and SER scheduling are not implemented here.
#define HLSL
typedef uint uint32_t;
#include "original/RaytracingHlslCompat.h"
#include "original/Star.hlsli"
#if MAX_RECURSION_DEPTH != 2
#error This specialization requires the pinned two-level recursion contract.
#endif
RaytracingAccelerationStructure Scene : register(t0);
RWTexture2D<float4> RenderTarget : register(u0);
ByteAddressBuffer IndicesCube : register(t1);
StructuredBuffer<Vertex> VerticesCube : register(t2);
ByteAddressBuffer IndicesTrunk : register(t3);
StructuredBuffer<Vertex> VerticesTrunk : register(t4);
ByteAddressBuffer IndicesLeaves : register(t5);
StructuredBuffer<Vertex> VerticesLeaves : register(t6);
ByteAddressBuffer IndicesBush : register(t7);
StructuredBuffer<Vertex> VerticesBush : register(t8);
ConstantBuffer<SceneConstantBuffer> g_sceneCB : register(b0);
Texture2D<float4> TrunkTexture : register(t9);
Texture2D<float4> BushTexture : register(t10);
SamplerState TrunkSampler : register(s0);
SamplerState BushSampler : register(s1);
RWStructuredBuffer<uint> AlkyholCounts : register(u20);
cbuffer AlkyholFrame : register(b20) {uint2 alkyholExtent; uint2 alkyholPadding;}
// All five original local-root CBs use white albedo. MaterialID is only read
// by the excluded SER hint helper, not by any included material function.
static const ObjectConstantBuffer g_cubeCB = {float4(1,1,1,1),0};
struct RayPayload {float4 color; uint recursionDepth;};
struct Ray {float3 Origin; float3 Direction;};
typedef BuiltInTriangleIntersectionAttributes MyAttributes;
static uint3 alkyholDispatch;
static float3 alkyholOrigin,alkyholDirection;
static float alkyholT;
static uint alkyholPrimitive,alkyholPrimaryMaterial;
#define DispatchRaysIndex() alkyholDispatch
#define DispatchRaysDimensions() uint3(alkyholExtent,1)
#define WorldRayOrigin() alkyholOrigin
#define WorldRayDirection() alkyholDirection
#define RayTCurrent() alkyholT
#define PrimitiveIndex() alkyholPrimitive
#include "helpers.generated.hlsli"
// Upstream returns float3 but assigns it to float. HLSL's implicit truncation
// selects x; make that exact conversion explicit without editing either body.
#define CalculateStarNest(...) (CalculateStarNest(__VA_ARGS__).x)
#include "miss.generated.hlsli"
#undef CalculateStarNest

void AlkyholBudgetFault(RaytracingAccelerationStructure acceleration,uint flags,uint mask,
    uint contribution,uint multiplier,uint missIndex,RayDesc ray,inout RayPayload payload)
{
    // Unreachable for the pinned depth-two program: TraceRadianceRay's original
    // guard returns black before a third trace. A violated contract fails the
    // frame; an exhausted budget never becomes a successful shader stub.
    InterlockedAdd(AlkyholCounts[10],1);
    payload.color=asfloat(uint4(0x7fc00000,0x7fc00000,0x7fc00000,0x7fc00000));
}
#define TraceRay AlkyholBudgetFault
#define TraceRadianceRay TraceRadianceRayD2
#define FloorClosestHitShader FloorClosestHitShaderD2
#define TrunkClosestHitShader TrunkClosestHitShaderD2
#define LeavesClosestHitShader LeavesClosestHitShaderD2
#define BushClosestHitShader BushClosestHitShaderD2
#define TCubeClosestHitShader TCubeClosestHitShaderD2
#include "radiance.generated.hlsli"
#include "hits.generated.hlsli"
#undef TraceRay
#undef TraceRadianceRay
#undef FloorClosestHitShader
#undef TrunkClosestHitShader
#undef LeavesClosestHitShader
#undef BushClosestHitShader
#undef TCubeClosestHitShader

uint AlkyholMaterial(uint instance) {return instance<441?0:instance<1402?1:instance<2363?2:instance<3324?3:4;}
void AlkyholTraceD2(RaytracingAccelerationStructure acceleration,uint flags,uint mask,
    uint contribution,uint multiplier,uint missIndex,RayDesc ray,inout RayPayload payload)
{
    if(payload.recursionDepth!=2) {AlkyholBudgetFault(acceleration,flags,mask,contribution,multiplier,missIndex,ray,payload);return;}
    if(alkyholPrimaryMaterial==0) InterlockedAdd(AlkyholCounts[6],1);
    float3 savedOrigin=alkyholOrigin,savedDirection=alkyholDirection;float savedT=alkyholT;uint savedPrimitive=alkyholPrimitive;
    alkyholOrigin=ray.Origin;alkyholDirection=ray.Direction;
    RayQuery<RAY_FLAG_FORCE_OPAQUE> query;query.TraceRayInline(acceleration,flags,mask,ray);while(query.Proceed()) {}
    if(query.CommittedStatus()==COMMITTED_TRIANGLE_HIT) {
        InterlockedAdd(AlkyholCounts[8],1);alkyholPrimitive=query.CommittedPrimitiveIndex();alkyholT=query.CommittedRayT();
        MyAttributes attr;attr.barycentrics=query.CommittedTriangleBarycentrics();
        switch(AlkyholMaterial(query.CommittedInstanceIndex())) {
        case 0:FloorClosestHitShaderD2(payload,attr);break;
        case 1:TCubeClosestHitShaderD2(payload,attr);break;
        case 2:TrunkClosestHitShaderD2(payload,attr);break;
        case 3:LeavesClosestHitShaderD2(payload,attr);break;
        case 4:BushClosestHitShaderD2(payload,attr);break;
        }
    } else {InterlockedAdd(AlkyholCounts[9],1);MyMissShader(payload);}
    // Original callers may read their ray builtins after a nested trace.
    alkyholOrigin=savedOrigin;alkyholDirection=savedDirection;alkyholT=savedT;alkyholPrimitive=savedPrimitive;
}
#define TraceRay AlkyholTraceD2
#define TraceRadianceRay TraceRadianceRayD1
#define FloorClosestHitShader FloorClosestHitShaderD1
#define TrunkClosestHitShader TrunkClosestHitShaderD1
#define LeavesClosestHitShader LeavesClosestHitShaderD1
#define BushClosestHitShader BushClosestHitShaderD1
#define TCubeClosestHitShader TCubeClosestHitShaderD1
#include "radiance.generated.hlsli"
#include "hits.generated.hlsli"
#undef TraceRay
#undef TraceRadianceRay
#undef FloorClosestHitShader
#undef TrunkClosestHitShader
#undef LeavesClosestHitShader
#undef BushClosestHitShader
#undef TCubeClosestHitShader

[numthreads(8,8,1)]
void AlkyholMain(uint3 id : SV_DispatchThreadID)
{
    if(any(id.xy>=alkyholExtent)) return;
    if(g_sceneCB.sortMode!=SORTMODE_OFF) {InterlockedAdd(AlkyholCounts[10],1);return;}
    alkyholDispatch=id;
    RayDesc ray;GenerateCameraRay(id.xy,ray.Origin,ray.Direction);ray.TMin=0.001;ray.TMax=10000;
    alkyholOrigin=ray.Origin;alkyholDirection=ray.Direction;
    RayPayload payload={float4(0,0,0,0),1};
    RayQuery<RAY_FLAG_FORCE_OPAQUE> query;query.TraceRayInline(Scene,0,~0,ray);while(query.Proceed()) {}
    if(query.CommittedStatus()==COMMITTED_TRIANGLE_HIT) {
        uint instance=query.CommittedInstanceIndex();alkyholPrimaryMaterial=AlkyholMaterial(instance);
        InterlockedAdd(AlkyholCounts[alkyholPrimaryMaterial],1);
        alkyholT=query.CommittedRayT();alkyholPrimitive=query.CommittedPrimitiveIndex();
        MyAttributes attr;attr.barycentrics=query.CommittedTriangleBarycentrics();
        switch(alkyholPrimaryMaterial) {
        case 0:FloorClosestHitShaderD1(payload,attr);break;
        case 1:InterlockedAdd(AlkyholCounts[7],1);TCubeClosestHitShaderD1(payload,attr);break;
        case 2:TrunkClosestHitShaderD1(payload,attr);break;
        case 3:LeavesClosestHitShaderD1(payload,attr);break;
        case 4:BushClosestHitShaderD1(payload,attr);break;
        }
    } else {InterlockedAdd(AlkyholCounts[5],1);MyMissShader(payload);}
    if(!all(isfinite(payload.color))) InterlockedAdd(AlkyholCounts[11],1);
    RenderTarget[id.xy]=payload.color;
}
