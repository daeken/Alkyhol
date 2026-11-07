// Copyright (c) Alkyhol contributors. MIT licensed.
#ifndef ALKYHOL_MINIENGINE_CONTEXT
#define ALKYHOL_MINIENGINE_CONTEXT
struct AlkyholHitRecord {
    float2 bary;float distance;uint mesh;
    uint primitive;float reflectivity;uint2 padding;
    float4 origin,direction;
};
cbuffer AlkyholSettings : register(b20) {uint alkyholWidth,alkyholHeight,alkyholMode,alkyholPass;}
cbuffer AlkyholProjection : register(b21) {float4x4 alkyholWorldToClip;}
#endif
