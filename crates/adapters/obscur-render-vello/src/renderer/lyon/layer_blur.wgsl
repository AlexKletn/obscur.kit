@group(0) @binding(0) var blurred: texture_2d<f32>;

@vertex
fn vertex(@builtin(vertex_index) corner: u32) -> @builtin(position) vec4<f32> {
    let corners = array(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return vec4(corners[corner], 0.0, 1.0);
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(blurred, vec2<i32>(position.xy), 0);
}
