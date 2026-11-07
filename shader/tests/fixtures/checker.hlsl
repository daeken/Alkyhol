// Copyright (c) Alkyhol contributors. MIT licensed.
// The shader bodies match the independent native D3D12 checker contract.
struct V { float4 position : SV_Position; };
V VSMain(uint id : SV_VertexID) {
    float2 p[3] = {float2(-1,-1),float2(-1,3),float2(3,-1)};
    V o; o.position = float4(p[id],0,1); return o;
}
float4 PSMain(V input) : SV_Target {
    uint x = (uint)input.position.x, y = (uint)input.position.y;
    return float4(x & 1, y & 1, ((x >> 1) + y) & 1, 1);
}
