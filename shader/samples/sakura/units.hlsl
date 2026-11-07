// Copyright (c) Alkyhol contributors. MIT licensed.
// Analytic checks call the exact source bodies used by the renderer. Dead ray
// entry points disappear, so this fixture also runs on hosted GPUs without RT.
#define AlkyholMain AlkyholUnusedSceneEntry
#include "ser_off.hlsl"
#undef AlkyholMain
RWStructuredBuffer<float4> AlkyholTestOutput : register(u30);
[numthreads(1,1,1)]
void AlkyholMain(uint3 id : SV_DispatchThreadID)
{
    if(id.x==0) AlkyholTestOutput[0]=float4(FresnelReflectanceSchlick(float3(0,0,-1),float3(0,0,1),float3(0.2,0.4,0.8)),1);
    if(id.x==1) AlkyholTestOutput[1]=float4(FresnelReflectanceSchlick(float3(1,0,0),float3(0,0,1),float3(0.2,0.4,0.8)),1);
    if(id.x==2) {
        float3 values[3]={float3(1,0,0),float3(0,1,0),float3(0,0,1)};
        MyAttributes attr;attr.barycentrics=float2(0.2,0.3);
        AlkyholTestOutput[2]=float4(HitAttribute(values,attr),1);
    }
    if(id.x==3) {
        float2 values[3]={float2(0.2,0.5),float2(0.6,0.7),float2(0.4,0.1)};
        MyAttributes attr;attr.barycentrics=float2(0.25,0.5);
        AlkyholTestOutput[3]=float4(HitAttribute(values,attr),0,1);
    }
    if(id.x==4) {
        alkyholOrigin=float3(1,2,3);alkyholDirection=float3(0,0,-1);alkyholT=2;
        AlkyholTestOutput[4]=float4(HitWorldPosition(),1);
    }
    if(id.x==5) {
        RayDesc ray={float3(0,0,0),0,float3(0,0,1),10};RayPayload payload={float4(0,0,0,0),3};
        AlkyholBudgetFault(Scene,0,255,0,1,0,ray,payload);
        AlkyholTestOutput[5]=float4(all(isnan(payload.color))?1:0,payload.recursionDepth,0,0);
    }
    if(id.x==6) {
        float3 direction=float3(1,0,0);
        AlkyholTestOutput[6]=float4(CalculateStarNest(direction*10,direction),1);
    }
}
