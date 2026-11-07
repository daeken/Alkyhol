#include <metal_stdlib>
using namespace metal;
struct VertexOutput { float4 position [[position]]; };
vertex VertexOutput vs_main(uint id [[vertex_id]]) {
    const float2 positions[3] = {float2(-1,-1), float2(-1,3), float2(3,-1)};
    return {float4(positions[id], 0, 1)};
}
fragment float4 fs_main(VertexOutput in [[stage_in]]) {
    uint x = uint(in.position.x), y = uint(in.position.y);
    return float4(x & 1, y & 1, ((x >> 1) + y) & 1, 1);
}
