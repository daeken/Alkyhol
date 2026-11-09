// Original MIT lowering of the XYZRHW|DIFFUSE fixed-function vertex contract.
// D3D9 samples pixel centers at integer coordinates; modern rasterizers at n+.5.
@group(0) @binding(0) var<uniform> viewport: vec4<f32>;
struct Output { @builtin(position) position: vec4<f32>, @location(0) color: vec4<f32> }
@vertex fn vs(@location(0) p: vec4<f32>, @location(1) bgra: vec4<f32>) -> Output {
    var o: Output;
    let w = 1.0 / p.w;
    let ndc = vec2<f32>(2.0 * (p.x + 0.5) / viewport.x - 1.0,
                       1.0 - 2.0 * (p.y + 0.5) / viewport.y);
    o.position = vec4<f32>(ndc * w, p.z * w, w);
    o.color = bgra.zyxw;
    return o;
}
@fragment fn fs(i: Output) -> @location(0) vec4<f32> { return i.color; }
