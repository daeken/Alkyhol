// Copyright (c) Alkyhol contributors. MIT licensed.
// Original DiffuseHit function bodies, with explicit existing-port extensions:
// nonfinite tangent repair, raster-style specular/view modes and diagnostics.
#define HLSL
static uint3 alkyholDispatch;
static float3 alkyholOrigin,alkyholDirection;
static float alkyholT;
static uint alkyholPrimitive,MaterialID,alkyholMaterial;
#define DispatchRaysIndex() alkyholDispatch
#define WorldRayOrigin() alkyholOrigin
#define WorldRayDirection() alkyholDirection
#define RayTCurrent() alkyholT
#define PrimitiveIndex() alkyholPrimitive
#include "original/ModelViewerRaytracing.h"
#include "original/RayTracingHlslCompat.h"
#include "context.hlsli"
StructuredBuffer<RayTraceMeshInfo> g_meshInfo : register(t1);
ByteAddressBuffer g_indices : register(t2);
ByteAddressBuffer g_attributes : register(t3);
Texture2D<float> texShadow : register(t4);
Texture2D<float> texSSAO : register(t5);
Texture2D<float4> normals : register(t13);
StructuredBuffer<AlkyholHitRecord> alkyholHits : register(t21);
StructuredBuffer<uint> alkyholMeshRoots : register(t25);
Texture2D<float4> alkyholDiffuse[27] : register(t0,space1);
Texture2D<float4> alkyholNormal[27] : register(t0,space2);
Texture2D<float4> alkyholSpecular[27] : register(t0,space3);
#define g_localTexture alkyholDiffuse[NonUniformResourceIndex(alkyholMaterial)]
#define g_localNormal alkyholNormal[NonUniformResourceIndex(alkyholMaterial)]
SamplerState g_s0 : register(s0);
SamplerComparisonState shadowSampler : register(s1);
RWStructuredBuffer<uint> alkyholCounts : register(u20);
RWTexture2D<float4> alkyholNormalOutput : register(u23);
#include "material_helpers.generated.hlsli"

static float2 alkyholUV,alkyholDX,alkyholDY;
static float3 alkyholEdge1,alkyholEdge2,alkyholSurfaceNormal,alkyholFrameNormal;
static float2 alkyholDelta1,alkyholDelta2;
static bool alkyholRepair,alkyholSpecEnabled;
static uint alkyholNormalizeIndex;
static float alkyholShadow;

float3 AlkyholNormalize(float3 v)
{
    uint call=++alkyholNormalizeIndex;
    if(call==1) alkyholFrameNormal=normalize(v);
    if(alkyholRepair && (call==2||call==3)) {
        float determinant=alkyholDelta1.x*alkyholDelta2.y-alkyholDelta1.y*alkyholDelta2.x;
        float3 t,b;
        if(abs(determinant)>1e-10) {
            t=(alkyholEdge1*alkyholDelta2.y-alkyholEdge2*alkyholDelta1.y)/determinant;
            b=(alkyholEdge2*alkyholDelta1.x-alkyholEdge1*alkyholDelta2.x)/determinant;
        } else {t=cross(abs(alkyholFrameNormal.y)<0.99?float3(0,1,0):float3(1,0,0),alkyholFrameNormal);b=cross(alkyholFrameNormal,t);}
        return normalize(call==2?t:b);
    }
    // The existing lit/specular extension follows the raster view convention;
    // original diffuse-only/secondary hit shading uses -WorldRayDirection.
    if(call==6&&alkyholSpecEnabled) return normalize(-v);
    return normalize(v);
}
float3 AlkyholMaterialLight(float3 diffuse,float3 specular,float mask,float gloss,float3 normal,float3 view,float3 light,float3 radiance)
{
    alkyholSurfaceNormal=normal;
    if(alkyholSpecEnabled) mask=alkyholSpecular[NonUniformResourceIndex(alkyholMaterial)].SampleGrad(g_s0,alkyholUV,alkyholDX,alkyholDY).g;
    return ApplyLightCommon(diffuse,specular,mask,gloss,normal,view,light,radiance);
}
void AlkyholShadowTrace(RaytracingAccelerationStructure acceleration,uint flags,uint mask,uint contribution,uint multiplier,uint missIndex,RayDesc ray,inout RayPayload payload)
{
    // The original Hit's only nested trace has SkipShading=true. Execute its
    // actual any-hit query and observed hit-distance payload contract.
    if(!payload.SkipShading) {InterlockedAdd(alkyholCounts[41],1);return;}
    RayQuery<RAY_FLAG_FORCE_OPAQUE> query;query.TraceRayInline(acceleration,flags,mask,ray);while(query.Proceed()) {}
    InterlockedAdd(alkyholCounts[35],1);
    if(query.CommittedStatus()==COMMITTED_TRIANGLE_HIT) {payload.RayHitT=query.CommittedRayT();alkyholShadow=0;InterlockedAdd(alkyholCounts[36],1);}
}
#define TraceRay AlkyholShadowTrace
#define normalize AlkyholNormalize
#define ApplyLightCommon AlkyholMaterialLight
#include "material_hit.generated.hlsli"
#undef TraceRay
#undef normalize
#undef ApplyLightCommon

void AlkyholPrepare(AlkyholHitRecord record)
{
    alkyholPrimitive=record.primitive%21845;
    MaterialID=alkyholMeshRoots[record.mesh]+record.primitive/21845;
    RayTraceMeshInfo info=g_meshInfo[MaterialID];alkyholMaterial=info.m_materialInstanceId;
    uint3 indices=Load3x16BitIndices(info.m_indexOffsetBytes+alkyholPrimitive*6);
    float3 weights=float3(1-record.bary.x-record.bary.y,record.bary);
    float3 p0=asfloat(g_attributes.Load3(info.m_positionAttributeOffsetBytes+indices.x*info.m_attributeStrideBytes));
    float3 p1=asfloat(g_attributes.Load3(info.m_positionAttributeOffsetBytes+indices.y*info.m_attributeStrideBytes));
    float3 p2=asfloat(g_attributes.Load3(info.m_positionAttributeOffsetBytes+indices.z*info.m_attributeStrideBytes));
    float2 u0=GetUVAttribute(info.m_uvAttributeOffsetBytes+indices.x*info.m_attributeStrideBytes);
    float2 u1=GetUVAttribute(info.m_uvAttributeOffsetBytes+indices.y*info.m_attributeStrideBytes);
    float2 u2=GetUVAttribute(info.m_uvAttributeOffsetBytes+indices.z*info.m_attributeStrideBytes);
    alkyholEdge1=p1-p0;alkyholEdge2=p2-p0;alkyholDelta1=u1-u0;alkyholDelta2=u2-u0;
    alkyholUV=u0*weights.x+u1*weights.y+u2*weights.z;
    float3 tangent=asfloat(g_attributes.Load3(info.m_tangentAttributeOffsetBytes+indices.x*info.m_attributeStrideBytes))*weights.x+asfloat(g_attributes.Load3(info.m_tangentAttributeOffsetBytes+indices.y*info.m_attributeStrideBytes))*weights.y+asfloat(g_attributes.Load3(info.m_tangentAttributeOffsetBytes+indices.z*info.m_attributeStrideBytes))*weights.z;
    float3 bitangent=asfloat(g_attributes.Load3(info.m_bitangentAttributeOffsetBytes+indices.x*info.m_attributeStrideBytes))*weights.x+asfloat(g_attributes.Load3(info.m_bitangentAttributeOffsetBytes+indices.y*info.m_attributeStrideBytes))*weights.y+asfloat(g_attributes.Load3(info.m_bitangentAttributeOffsetBytes+indices.z*info.m_attributeStrideBytes))*weights.z;
    alkyholRepair=!all(isfinite(tangent))||!all(isfinite(bitangent))||dot(tangent,tangent)<1e-12||dot(bitangent,bitangent)<1e-12;
    alkyholOrigin=record.origin.xyz;alkyholDirection=record.direction.xyz;alkyholT=record.distance;
    float3 position=record.origin.xyz+record.direction.xyz*record.distance;
    float3 plane=normalize(cross(p2-p0,p1-p0));
    float3 originX,dirX,originY,dirY;GenerateCameraRay(alkyholDispatch.xy+uint2(1,0),originX,dirX);GenerateCameraRay(alkyholDispatch.xy+uint2(0,1),originY,dirY);
    float3 bx=BarycentricCoordinates(RayPlaneIntersection(position,plane,originX,dirX),p0,p1,p2);
    float3 by=BarycentricCoordinates(RayPlaneIntersection(position,plane,originY,dirY),p0,p1,p2);
    float3x2 uvMatrix=float3x2(u0,u1,u2);alkyholDX=mul(bx,uvMatrix)-alkyholUV;alkyholDY=mul(by,uvMatrix)-alkyholUV;
}
[numthreads(8,8,1)]
void AlkyholMain(uint3 id:SV_DispatchThreadID)
{
    if(id.x>=alkyholWidth||id.y>=alkyholHeight) return;
    uint index=id.y*alkyholWidth+id.x+alkyholPass*alkyholWidth*alkyholHeight;
    AlkyholHitRecord record=alkyholHits[index];if(record.mesh==0xffffffff)return;
    alkyholDispatch=id;alkyholSpecEnabled=alkyholPass==0&&alkyholMode!=6;
    AlkyholPrepare(record);alkyholNormalizeIndex=0;alkyholShadow=1;
    BuiltInTriangleIntersectionAttributes attr;attr.barycentrics=record.bary;
    RayPayload payload;payload.SkipShading=false;payload.RayHitT=FLT_MAX;
    Hit(payload,attr);
    if(alkyholNormalizeIndex!=6)InterlockedAdd(alkyholCounts[41],1);
    if(alkyholRepair) InterlockedAdd(alkyholCounts[39],1);
    if(alkyholPass==0) {
        float reflectivity=alkyholMaterial==10?alkyholSpecular[NonUniformResourceIndex(alkyholMaterial)].SampleGrad(g_s0,alkyholUV,alkyholDX,alkyholDY).g:0;
        alkyholNormalOutput[id.xy]=float4(alkyholSurfaceNormal,reflectivity);
        if(alkyholMode==4)g_screenOutput[id.xy]=float4(alkyholShadow.xxx,1);
    }
    if(!all(isfinite(g_screenOutput[id.xy]))) InterlockedAdd(alkyholCounts[41],1);
}
