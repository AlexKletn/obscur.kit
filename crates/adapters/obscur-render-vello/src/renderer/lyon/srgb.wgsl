
// Transfer functions operate on straight RGB; attachments carry premultiplied RGB.
fn encode_srgb(linear: vec3<f32>) -> vec3<f32> {
    let rgb = clamp(linear, vec3(0.0), vec3(1.0));
    return select(12.92 * rgb, 1.055 * pow(rgb, vec3(1.0 / 2.4)) - 0.055, rgb > vec3(0.0031308));
}

fn decode_srgb(encoded: vec3<f32>) -> vec3<f32> {
    let rgb = clamp(encoded, vec3(0.0), vec3(1.0));
    return select(rgb / 12.92, pow((rgb + 0.055) / 1.055, vec3(2.4)), rgb > vec3(0.04045));
}

fn encode_premul_srgb(color: vec4<f32>) -> vec4<f32> {
    return vec4(encode_srgb(color.rgb / max(color.a, 1e-15)) * color.a, color.a);
}

fn decode_premul_srgb(color: vec4<f32>) -> vec4<f32> {
    return vec4(decode_srgb(color.rgb / max(color.a, 1e-15)) * color.a, color.a);
}
