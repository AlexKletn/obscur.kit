struct Transform {
    x: vec4<f32>,
    y: vec4<f32>,
};

@group(0) @binding(0) var<uniform> transforms: array<Transform, 256>;
@group(0) @binding(1) var<uniform> gradient_data: array<vec4<f32>, 1024>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) shadow: vec4<f32>,
    @location(2) @interpolate(flat) parameters: vec2<f32>,
    @location(3) local: vec2<f32>,
    @location(4) @interpolate(flat) gradient: u32,
};

@vertex
fn vertex(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) shadow: vec4<f32>,
    @location(3) parameters: vec2<f32>,
    @location(4) transform_index: u32,
    @location(5) gradient_index: u32,
) -> VertexOutput {
    let transform = transforms[transform_index];
    let point = vec3<f32>(position, 1.0);
    let clip = vec2<f32>(dot(transform.x.xyz, point), dot(transform.y.xyz, point));
    return VertexOutput(vec4<f32>(clip, 0.0, 1.0), color * transform.x.w, shadow, parameters, position, gradient_index);
}

// Solve |point - t * center_delta|² = (radius + t * radius_delta)².
fn radial_parameter(point: vec2<f32>, delta: vec2<f32>, radius: vec2<f32>) -> vec2<f32> {
    let a = dot(delta, delta) - radius.y * radius.y;
    let b = -2.0 * (dot(point, delta) + radius.x * radius.y);
    let c = dot(point, point) - radius.x * radius.x;
    if a == 0.0 {
        if b == 0.0 { return vec2(0.0); }
        let t = -c / b;
        return vec2(t, select(0.0, 1.0, radius.x + t * radius.y > 0.0));
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 { return vec2(0.0); }
    let root = sqrt(discriminant);
    let q = -0.5 * (b + select(-root, root, b >= 0.0));
    var t0 = -b / (2.0 * a);
    var t1 = t0;
    if q != 0.0 {
        t0 = q / a;
        t1 = c / q;
    }
    // OpenType paints the largest parameter whose circle has positive radius.
    let high = max(t0, t1);
    if radius.x + high * radius.y > 0.0 { return vec2(high, 1.0); }
    let low = min(t0, t1);
    return vec2(low, select(0.0, 1.0, radius.x + low * radius.y > 0.0));
}

// Font stops are premultiplied linear RGB. Scene gradients interpolate in Oklab.
fn gradient_color(index: u32, point: vec2<f32>, glyph_origin: vec2<f32>) -> vec4<f32> {
    let header = gradient_data[index];
    let geometry = gradient_data[index + 1u];
    var local = point - geometry.xy;
    var first_stop = index + 2u;
    if header.w != 0.0 {
        let origin = vec3(point - glyph_origin, 1.0);
        local = vec2(dot(gradient_data[index + 2u].xyz, origin),
                     dot(gradient_data[index + 3u].xyz, origin)) - geometry.xy;
        first_stop = index + 5u;
    }
    var t: f32;
    switch u32(header.x) {
        case 0u: { t = dot(local, geometry.zw); }
        case 1u: { t = length(local * geometry.zw); }
        case 3u: {
            let radial = radial_parameter(local, geometry.zw, gradient_data[index + 4u].xy);
            if radial.y == 0.0 { return vec4(0.0); }
            t = radial.x;
        }
        case 4u: {
            // Skrifa normalizes the stops against clockwise angles in font space.
            var turn = 1.0;
            if any(local != vec2(0.0)) {
                turn -= fract(atan2(local.y, local.x) / 6.28318530718);
            }
            if geometry.z == geometry.w {
                t = select(0.0, 1.0, turn >= geometry.z);
            } else {
                t = (turn - geometry.z) / (geometry.w - geometry.z);
            }
        }
        default: {
            t = 0.0;
            if any(local != vec2(0.0)) {
                t = fract((atan2(local.y, local.x) - geometry.z) / 6.28318530718);
            }
        }
    }
    switch u32(header.y) {
        case 1u: { t = fract(t); }
        case 2u: { t = 1.0 - abs(2.0 * fract(t * 0.5) - 1.0); }
        default: { t = clamp(t, 0.0, 1.0); }
    }
    var left_offset = gradient_data[first_stop].x;
    var sample = gradient_data[first_stop + 1u];
    for (var stop = 1u; stop < u32(header.z); stop += 1u) {
        let next_stop = first_stop + 2u * stop;
        let right_offset = gradient_data[next_stop].x;
        let right = gradient_data[next_stop + 1u];
        if t < right_offset {
            if t > left_offset {
                sample = mix(sample, right, (t - left_offset) / (right_offset - left_offset));
            }
            break;
        }
        left_offset = right_offset;
        sample = right;
    }
    if header.w != 0.0 {
        return sample;
    }
    if sample.w <= 0.0 {
        return vec4<f32>(0.0);
    }
    let lab = sample.xyz / sample.w;
    let lms = vec3(
        dot(lab, vec3(1.0, 0.3963377774, 0.2158037573)),
        dot(lab, vec3(1.0, -0.1055613458, -0.0638541728)),
        dot(lab, vec3(1.0, -0.0894841775, -1.2914855480)),
    );
    let cube = lms * lms * lms;
    let linear = clamp(vec3(
        dot(cube, vec3(4.0767416621, -3.3077115913, 0.2309699292)),
        dot(cube, vec3(-1.2684380046, 2.6097574011, -0.3413193965)),
        dot(cube, vec3(-0.0041960863, -0.7034186147, 1.7076147010)),
    ), vec3(0.0), vec3(1.0));
    return vec4(encode_srgb(linear) * sample.w, sample.w);
}

// Approximate the blurred edge of a rounded rectangle in its own coordinate space.
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // A negative fourth lane marks contours targeting an isolated linear glyph.
    let linear_target = in.shadow.w < 0.0;
    if in.gradient != 0u {
        let color = gradient_color(in.gradient, in.local, in.shadow.xy) * in.color.a;
        let font_gradient = gradient_data[in.gradient].w != 0.0;
        if font_gradient && !linear_target { return encode_premul_srgb(color); }
        if !font_gradient && linear_target { return decode_premul_srgb(color); }
        return color;
    }
    let sigma = in.parameters.y;
    if sigma == 0.0 {
        if linear_target { return decode_premul_srgb(in.color); }
        return in.color;
    }
    let radius = in.parameters.x;
    let local = abs(in.shadow.xy) - in.shadow.zw + vec2<f32>(radius);
    let distance = length(max(local, vec2<f32>(0.0))) + min(max(local.x, local.y), 0.0) - radius;
    let coverage = 1.0 / (1.0 + exp(1.702 * distance / abs(sigma)));
    return in.color * select(coverage, 1.0 - coverage, sigma < 0.0);
}

@group(1) @binding(0) var image_texture: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;

fn luminance_texel(point: vec2<i32>, last: vec2<i32>) -> f32 {
    let pixel = textureLoad(image_texture, clamp(point, vec2(0), last), 0);
    return dot(decode_premul_srgb(pixel).rgb, vec3(0.2126, 0.7152, 0.0722));
}

fn luminance_coverage(uv: vec2<f32>) -> f32 {
    let size = textureDimensions(image_texture);
    let position = uv * vec2<f32>(size) - 0.5;
    let point = vec2<i32>(floor(position));
    let weight = fract(position);
    let last = vec2<i32>(size) - 1;
    // Convert each texel before filtering the coverage field.
    let top = mix(luminance_texel(point, last), luminance_texel(point + vec2(1, 0), last), weight.x);
    let bottom = mix(luminance_texel(point + vec2(0, 1), last), luminance_texel(point + vec2(1), last), weight.x);
    return mix(top, bottom, weight.y);
}

@fragment
fn fragment_image(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample each tile with clamped filtering, like an independently drawn
    // copy. Fract on a repeated axis handles partial and negative tile indices.
    let uv = select(in.shadow.xy, fract(in.shadow.xy), in.parameters > vec2(0.0));
    if in.color.r == 0.0 {
        return vec4(luminance_coverage(uv)) * in.color.a;
    }
    return textureSampleLevel(image_texture, image_sampler, uv, 0.0) * in.color.a;
}
