// Adapted from Microsoft Raytracing.hlsl and Star.hlsli, pinned in upstream.json.
// Copyright (c) Microsoft; Copyright (c) Pablo Roman Andrioli. MIT, see licenses/.
#include <metal_stdlib>
using namespace metal;
struct Vertex { packed_float3 position; packed_float3 normal; packed_float2 uv; };
constant uint cube_indices[36]={3,1,0,2,1,3,6,4,5,7,4,6,11,9,8,10,9,11,14,12,13,15,12,14,19,17,16,18,17,19,22,20,21,23,20,22};
float3 interpolate_normal(device const Vertex* v,uint3 indices,float2 b) {
    float3 a=v[indices.x].normal,c=v[indices.y].normal,d=v[indices.z].normal;
    return a+b.x*(c-a)+b.y*(d-a);
}
float star_density(float3 direction) {
    float3 sky=direction*10.0f;float density=0,fade=1;
    for(int s=0;s<200;++s) {
        float3 p=sky+direction*(s*0.05f);p=abs(fmod(p,2.0f)-1.0f);
        float pa=0,a=0;
        for(int i=0;i<50;++i) {float inv=1.0f/dot(p,p);p=abs(p)*inv-0.53f;float len=length(p);a+=abs(len-pa);pa=len;}
        density+=a*fade;fade*=0.9f;
    }
    return smoothstep(0.1f,0.9f,saturate(density/(200*50*0.12f)));
}
float4 miss_colour(float3 direction,float3 light) {
    float3 d=normalize(direction),sky=d*10.0f;
    float density=star_density(d);float3 glow=mix(float3(.4,.6,1),float3(1,.6,.8),density);
    float ld=saturate(dot(d,normalize(light-sky)));glow*=mix(.8f+.2f*ld,4.0f,density);
    return float4(mix(float3(.05,.02,.08),glow,density*1.3f),1);
}
float3 material_normal(uint material,uint primitive,float2 b,uint leaf_vertices,
                       device const Vertex* cube,device const Vertex* trunk,
                       device const Vertex* leaves,device const Vertex* bush) {
    uint base=primitive*3;uint3 indices=uint3(base,base+1,base+2);
    if(material==0)return interpolate_normal(cube,indices,b);
    if(material==1)return interpolate_normal(trunk,uint3(cube_indices[base],cube_indices[base+1],cube_indices[base+2]),b);
    if(material==2) {
        // The upstream trunk shader reads the leaf index SRV, then trunk normals.
        // Leaf indices are sequential; descriptor-table reads outside its bound return zero.
        if(base+2>=leaf_vertices)indices=uint3(0);
        return interpolate_normal(trunk,indices,b);
    }
    return interpolate_normal(material==3?leaves:bush,indices,b);
}
float3 schlick(float3 incident,float3 normal,float3 f0) {
    float cosi=saturate(dot(-incident,normal));
    return f0+(1-f0)*pow(1-cosi,5.0f);
}
float3 triplanar_weights(float3 normal) {
    return normalize(max(abs(normal),float3(.00001)));
}
float3 petal_colour(uint primitive) {
    return primitive%3==0?float3(.8,.7,.9):primitive%3==1?float3(1,.8,.9):float3(.85,.4,.6);
}
