// Copyright (c) Alkyhol contributors. MIT licensed.
// Alkyhol-specific views; no original sample shader exists for these controls.
#define HLSL
#include "original/RayTracingHlslCompat.h"
#include "context.hlsli"
StructuredBuffer<RayTraceMeshInfo> g_meshInfo : register(t1);
ByteAddressBuffer g_indices : register(t2);
ByteAddressBuffer g_attributes : register(t3);
StructuredBuffer<AlkyholHitRecord> alkyholHits : register(t21);
StructuredBuffer<uint> alkyholMeshRoots : register(t25);
RWTexture2D<float4> g_screenOutput : register(u2);
uint3 Load3x16BitIndices(uint offsetBytes) {
    uint offset=offsetBytes&~3;uint2 words=g_indices.Load2(offset);
    return offset==offsetBytes?uint3(words.x&65535,words.x>>16,words.y&65535):uint3(words.x>>16,words.y&65535,words.y>>16);
}
[numthreads(8,8,1)]
void AlkyholMain(uint3 id:SV_DispatchThreadID) {
    if(id.x>=alkyholWidth||id.y>=alkyholHeight)return;
    AlkyholHitRecord hit=alkyholHits[id.y*alkyholWidth+id.x];if(hit.mesh==0xffffffff)return;
    RayTraceMeshInfo info=g_meshInfo[alkyholMeshRoots[hit.mesh]+hit.primitive/21845];uint3 indices=Load3x16BitIndices(info.m_indexOffsetBytes+(hit.primitive%21845)*6);
    float3 a=asfloat(g_attributes.Load3(info.m_normalAttributeOffsetBytes+indices.x*56));float3 b=asfloat(g_attributes.Load3(info.m_normalAttributeOffsetBytes+indices.y*56));float3 c=asfloat(g_attributes.Load3(info.m_normalAttributeOffsetBytes+indices.z*56));
    float3 normal=normalize(a+hit.bary.x*(b-a)+hit.bary.y*(c-a));
    float3 colour=alkyholMode==8?normal*0.5+0.5:alkyholMode==9?(1./(1.+hit.distance/1000.)).xxx:0.72*(0.4+0.6*saturate(dot(normal,-hit.direction.xyz))).xxx;
    g_screenOutput[id.xy]=float4(colour,1);
}
