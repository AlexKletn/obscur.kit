@group(0) @binding(0) var group_color: texture_multisampled_2d<f32>;

struct ColorFilter {
    r: vec4<f32>,
    g: vec4<f32>,
    b: vec4<f32>,
    a: vec4<f32>,
    bias: vec4<f32>,
    group: vec4<f32>,
};

@group(0) @binding(1) var<uniform> filters: array<ColorFilter, 128>;

struct GroupVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) parameters: u32,
    @location(1) @interpolate(flat) blend_mode: u32,
};

@vertex
fn vertex(@builtin(vertex_index) corner: u32, @builtin(instance_index) parameters: u32) -> GroupVertex {
    let corners = array(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return GroupVertex(vec4(corners[corner % 3u], 0.0, 1.0), parameters, corner / 3u);
}

fn apply_group(color: vec4<f32>, parameters: u32) -> vec4<f32> {
    if (parameters & 0x80000000u) == 0u {
        return color * bitcast<f32>(parameters);
    }
    if color.a <= 0.0 {
        return vec4(0.0);
    }
    let matrix = filters[parameters & 0x7fffffffu];
    let straight = vec4(color.rgb / color.a, color.a);
    let transformed = clamp(
        vec4(dot(matrix.r, straight), dot(matrix.g, straight),
             dot(matrix.b, straight), dot(matrix.a, straight)) + matrix.bias,
        vec4(0.0), vec4(1.0),
    );
    let alpha = transformed.a * matrix.group.x;
    return vec4(transformed.rgb * alpha, alpha);
}

@fragment
fn fragment(input: GroupVertex, @builtin(sample_index) sample: u32) -> @location(0) vec4<f32> {
    var point = vec2<i32>(input.position.xy);
    if input.blend_mode >= 2u {
        let origin = input.blend_mode - 2u;
        var offset = vec2<i32>(i32(origin & 0x3fffu), i32((origin >> 14u) & 0x3fffu));
        if (origin & (1u << 28u)) != 0u {
            offset -= vec2<i32>(8192);
        }
        point -= offset;
    }
    let color = apply_group(textureLoad(group_color, point, i32(sample)), input.parameters);
    if input.blend_mode == 1u { return encode_premul_srgb(color); }
    return color;
}

@fragment
fn resolve_filtered(input: GroupVertex) -> @location(0) vec4<f32> {
    let point = vec2<i32>(input.position.xy);
    var result = vec4(0.0);
    for (var sample = 0; sample < 4; sample += 1) {
        result += apply_group(textureLoad(group_color, point, sample), input.parameters);
    }
    return result * 0.25;
}
