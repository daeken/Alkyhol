#include <metal_stdlib>
using namespace metal;
struct VertexOutput { float4 position [[position]]; };
vertex VertexOutput vs_main(uint id [[vertex_id]]) {
    const float2 positions[3] = {float2(-0.75,-0.75), float2(0.75,-0.75), float2(0,0.75)};
    return {float4(positions[id], 0, 1)};
}
fragment float4 fs_main(VertexOutput in [[stage_in]]) { return float4(1,0,0,1); }
