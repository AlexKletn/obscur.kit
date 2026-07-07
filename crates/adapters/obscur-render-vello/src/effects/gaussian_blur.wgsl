// Separable gaussian blur + premultiplied composite (spec §5.2).
//
// Three entry points share a fullscreen-triangle / quad vertex stage:
//   * `blur_fs`      — one separable gaussian pass (horizontal or vertical),
//                      the direction and radius supplied by `BlurParams`.
//   * `composite_vs`/`composite_fs` — draw the blurred texture as a quad at a
//                      target-space NDC rect, sampling it 0..1, premultiplied-over.
//
// All textures are premultiplied RGBA8 (Vello's output encoding); the blur is a
// normalized weighted sum of premultiplied texels, which stays premultiplied, and
// the composite blends premultiplied-over (`src * 1 + dst * (1 - src.a)`).

struct BlurParams {
    // Per-tap UV step for the blur direction: (1/width, 0) horizontal, (0, 1/height)
    // vertical.
    direction: vec2<f32>,
    // Blur radius in texels; also the gaussian standard deviation (CSS `blur()`).
    radius: f32,
    _pad: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_smp: sampler;
@group(0) @binding(2) var<uniform> blur: BlurParams;

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
fn blur_fs(in: VsOut) -> @location(0) vec4<f32> {
    let sigma = max(blur.radius, 0.0001);
    // Gaussian support of 3 sigma each side captures ~99.7% of the kernel.
    let support = i32(ceil(blur.radius * 3.0));
    var accum = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var i = -support; i <= support; i = i + 1) {
        let fi = f32(i);
        let w = exp(-(fi * fi) / (2.0 * sigma * sigma));
        let uv = in.uv + blur.direction * fi;
        accum = accum + textureSampleLevel(src_tex, src_smp, uv, 0.0) * w;
        wsum = wsum + w;
    }
    return accum / max(wsum, 0.0001);
}

struct CompositeParams {
    // Target-space NDC rect (x0, y_top, x1, y_bottom); uv (0,0) maps to (x0, y_top).
    rect: vec4<f32>,
};

@group(0) @binding(0) var comp_tex: texture_2d<f32>;
@group(0) @binding(1) var comp_smp: sampler;
@group(0) @binding(2) var<uniform> comp: CompositeParams;

// Two triangles covering `comp.rect`, sampling the source texture over UV 0..1.
@vertex
fn composite_vs(@builtin(vertex_index) vid: u32) -> VsOut {
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let uv = uvs[vid];
    var out: VsOut;
    out.uv = uv;
    let x = mix(comp.rect.x, comp.rect.z, uv.x);
    let y = mix(comp.rect.y, comp.rect.w, uv.y);
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment
fn composite_fs(in: VsOut) -> @location(0) vec4<f32> {
    return textureSampleLevel(comp_tex, comp_smp, in.uv, 0.0);
}
