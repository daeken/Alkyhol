@vertex fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(-1.0, 3.0), vec2(3.0, -1.0));
    return vec4(positions[id], 0.0, 1.0);
}
@fragment fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let x = u32(position.x);
    let y = u32(position.y);
    return vec4(f32(x & 1u), f32(y & 1u), f32(((x >> 1u) + y) & 1u), 1.0);
}
