// A 4×5 affine color-matrix pass (CSS Filter Effects §; spec §5.2).
//
// One fullscreen fragment pass applies `rgba' = M · rgba + bias` (per channel),
// clamped to 0..1, to a sampled source texture. The matrix is the coordinator's
// folded color-op chain (grayscale/brightness/contrast/saturate/sepia/invert/
// hue-rotate/opacity composed into a single affine). Vello's texture output is
// STRAIGHT (non-premultiplied) alpha, which is exactly the space CSS color
// filters are defined over, so the transform is applied directly to the texels.

struct ColorMatrixParams {
    // The four rows of the linear 4×4 part: out.channel_i = dot(row_i, rgba).
    r0: vec4<f32>,
    r1: vec4<f32>,
    r2: vec4<f32>,
    r3: vec4<f32>,
    // The per-channel additive bias.
    bias: vec4<f32>,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_smp: sampler;
@group(0) @binding(2) var<uniform> cm: ColorMatrixParams;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle covering the whole target, UV in [0, 1].
@vertex
fn fullscreen_vs(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vid << 1u) & 2u);
    let y = f32(vid & 2u);
    out.uv = vec2<f32>(x, y);
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

@fragment
fn color_matrix_fs(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSampleLevel(src_tex, src_smp, in.uv, 0.0);
    let out = vec4<f32>(
        dot(cm.r0, c) + cm.bias.x,
        dot(cm.r1, c) + cm.bias.y,
        dot(cm.r2, c) + cm.bias.z,
        dot(cm.r3, c) + cm.bias.w,
    );
    return clamp(out, vec4<f32>(0.0), vec4<f32>(1.0));
}
