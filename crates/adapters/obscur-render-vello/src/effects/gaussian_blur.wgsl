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
    // Optional rounded-rect mask (backdrop-filter frosts only the panel shape). In
    // the QUAD's own pixel space (uv * size.xy): the panel border box x0,y0,x1,y1.
    panel: vec4<f32>,
    // Per-corner radii of the panel, in quad pixels: (top-left, top-right,
    // bottom-right, bottom-left).
    radii: vec4<f32>,
    // (quad_width_px, quad_height_px, mask_enabled (0/1), aa_half_width_px).
    size_flag: vec4<f32>,
    // (premultiply_src (0/1), _, _, _). Vello's texture output is STRAIGHT alpha,
    // so a segment sampled straight from a Vello frame is premultiplied here before
    // the premultiplied-over blend; a source already premultiplied (a blurred copy
    // of the accumulating frame) sets this to 0.
    flags2: vec4<f32>,
};

@group(0) @binding(0) var comp_tex: texture_2d<f32>;
@group(0) @binding(1) var comp_smp: sampler;
@group(0) @binding(2) var<uniform> comp: CompositeParams;

// Coverage (1 inside, 0 outside, smooth across the edge) of a per-corner rounded
// rectangle `rect` (x0,y0,x1,y1) at pixel `p`, radii = (tl, tr, br, bl), `aa` the
// half-width of the antialiased edge. Standard rounded-box SDF, the corner radius
// chosen by which quadrant `p` falls in relative to the rect centre.
fn rounded_rect_coverage(p: vec2<f32>, rect: vec4<f32>, radii: vec4<f32>, aa: f32) -> f32 {
    let center = 0.5 * (rect.xy + rect.zw);
    let half = 0.5 * (rect.zw - rect.xy);
    let q = p - center;
    // Quadrant → corner radius. y < 0 is the top half (pixel y grows downward).
    var r = radii.x; // top-left
    if (q.x >= 0.0 && q.y < 0.0) {
        r = radii.y; // top-right
    } else if (q.x >= 0.0 && q.y >= 0.0) {
        r = radii.z; // bottom-right
    } else if (q.x < 0.0 && q.y >= 0.0) {
        r = radii.w; // bottom-left
    }
    r = max(min(r, min(half.x, half.y)), 0.0);
    let d = abs(q) - half + vec2<f32>(r, r);
    let dist = min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0, 0.0))) - r;
    // dist < 0 inside the shape; fade over [-aa, aa].
    return 1.0 - smoothstep(-aa, aa, dist);
}

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
    var color = textureSampleLevel(comp_tex, comp_smp, in.uv, 0.0);
    // Vello output is STRAIGHT alpha; premultiply it so the pipeline's
    // premultiplied-over blend (`src * 1 + dst * (1 - src.a)`) composites it
    // correctly. A premultiplied source (blurred frame copy) skips this (flag == 0);
    // for opaque pixels (a == 1) it is a no-op, so the shipped `filter: blur`
    // composite stays byte-identical.
    if (comp.flags2.x > 0.5) {
        color = vec4<f32>(color.rgb * color.a, color.a);
    }
    // Rounded-rect mask (backdrop-filter): scale the premultiplied color by the
    // panel-shape coverage so only the rounded area frosts. Disabled (flag == 0)
    // for the plain `filter: blur` composite, where coverage is a no-op × 1.
    if (comp.size_flag.z > 0.5) {
        let px = in.uv * comp.size_flag.xy;
        let cov = rounded_rect_coverage(px, comp.panel, comp.radii, comp.size_flag.w);
        color = color * cov;
    }
    return color;
}
