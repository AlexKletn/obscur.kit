
@group(1) @binding(0) var backdrop_color: texture_multisampled_2d<f32>;

fn screen(backdrop: f32, source: f32) -> f32 {
    return backdrop + source - backdrop * source;
}

fn hard_light(backdrop: f32, source: f32) -> f32 {
    if source <= 0.5 {
        return backdrop * (2.0 * source);
    }
    return screen(backdrop, 2.0 * source - 1.0);
}

// The discriminants come from peniko::Mix through the shared scene converter.
fn mix_channel(b: f32, s: f32, mode: u32) -> f32 {
    switch mode {
        case 1u: { return b * s; } // Multiply
        case 2u: { return screen(b, s); }
        case 3u: { return hard_light(s, b); } // Overlay
        case 4u: { return min(b, s); } // Darken
        case 5u: { return max(b, s); } // Lighten
        case 6u: { // Color dodge
            if b == 0.0 { return 0.0; }
            if s == 1.0 { return 1.0; }
            return min(1.0, b / (1.0 - s));
        }
        case 7u: { // Color burn
            if b == 1.0 { return 1.0; }
            if s == 0.0 { return 0.0; }
            return 1.0 - min(1.0, (1.0 - b) / s);
        }
        case 8u: { return hard_light(b, s); }
        case 9u: { // Soft light
            if s <= 0.5 { return b - (1.0 - 2.0 * s) * b * (1.0 - b); }
            let d = select(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, b <= 0.25);
            return b + (2.0 * s - 1.0) * (d - b);
        }
        case 10u: { return abs(b - s); } // Difference
        case 11u: { return b + s - 2.0 * b * s; } // Exclusion
        default: { return s; }
    }
}

fn luminosity(color: vec3<f32>) -> f32 {
    return dot(color, vec3(0.3, 0.59, 0.11));
}

fn saturation(color: vec3<f32>) -> f32 {
    return max(max(color.r, color.g), color.b) - min(min(color.r, color.g), color.b);
}

fn set_saturation(color: vec3<f32>, value: f32) -> vec3<f32> {
    let spread = saturation(color);
    if spread == 0.0 { return vec3(0.0); }
    return (color - min(min(color.r, color.g), color.b)) * (value / spread);
}

fn set_luminosity(color: vec3<f32>, value: f32) -> vec3<f32> {
    var adjusted = color + (value - luminosity(color));
    let low = min(min(adjusted.r, adjusted.g), adjusted.b);
    let high = max(max(adjusted.r, adjusted.g), adjusted.b);
    if low < 0.0 {
        adjusted = value + (adjusted - value) * (value / (value - low));
    }
    if high > 1.0 {
        adjusted = value + (adjusted - value) * ((1.0 - value) / (high - value));
    }
    return adjusted;
}

fn mix_color(b: vec3<f32>, s: vec3<f32>, mode: u32) -> vec3<f32> {
    switch mode {
        case 12u: { return set_luminosity(set_saturation(s, saturation(b)), luminosity(b)); }
        case 13u: { return set_luminosity(set_saturation(b, saturation(s)), luminosity(b)); }
        case 14u: { return set_luminosity(s, luminosity(b)); }
        case 15u: { return set_luminosity(b, luminosity(s)); }
        default: { return vec3(mix_channel(b.r, s.r, mode), mix_channel(b.g, s.g, mode), mix_channel(b.b, s.b, mode)); }
    }
}

@fragment
fn blend_fragment(input: GroupVertex, @builtin(sample_index) sample: u32) -> @location(0) vec4<f32> {
    let point = vec2<i32>(input.position.xy);
    let source = apply_group(textureLoad(group_color, point, i32(sample)), input.parameters);
    let backdrop = textureLoad(backdrop_color, point, i32(sample));
    let s = source.rgb / max(source.a, 1e-15);
    let b = backdrop.rgb / max(backdrop.a, 1e-15);
    let mode = input.blend_mode >> 8u;
    let mixed = mix_color(b, s, mode);
    let paint = vec4(mix(s, mixed, backdrop.a) * source.a, source.a);

    // The low byte is peniko::Compose.
    switch input.blend_mode & 255u {
        case 0u: { return vec4(0.0); } // Clear
        case 1u: { return paint; } // Copy
        case 2u: { return backdrop; } // Destination
        case 4u: { return backdrop + paint * (1.0 - backdrop.a); } // Destination over
        case 5u: { return paint * backdrop.a; } // Source in
        case 6u: { return backdrop * source.a; } // Destination in
        case 7u: { return paint * (1.0 - backdrop.a); } // Source out
        case 8u: { return backdrop * (1.0 - source.a); } // Destination out
        case 9u: { return paint * backdrop.a + backdrop * (1.0 - source.a); } // Source atop
        case 10u: { return backdrop * source.a + paint * (1.0 - backdrop.a); } // Destination atop
        case 11u: { return paint * (1.0 - backdrop.a) + backdrop * (1.0 - source.a); } // XOR
        case 12u: { return min(vec4(1.0), paint + backdrop); } // Plus
        default: { return paint + backdrop * (1.0 - source.a); }
    }
}
