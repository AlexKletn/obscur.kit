// Present blit: Vello's straight-alpha output onto a premultiplied surface.
//
// Vello writes STRAIGHT alpha (`rgb` is the colour, `a` is beside it). Every
// alpha mode a Wayland swapchain offers is `Opaque` or `PreMultiplied`, and
// under the latter the compositor computes `dst = src.rgb + (1 - src.a) * dst`
// WITHOUT scaling `src.rgb` first. Copying straight alpha through verbatim
// therefore paints a translucent surface as an opaque one, brightened by
// whatever was behind it — which is exactly what a plain
// `wgpu::util::TextureBlitter` does.
//
// So the multiply happens here, in the encoded space the surface stores: that
// is what "premultiplied" means for an 8-bit sRGB buffer, and what every
// browser compositor does with the same data.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// One oversized triangle rather than two triangles: no shared edge to seam, and
// no vertex buffer to bind.
@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    out.uv = uv;
    out.position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let colour = textureSample(source, source_sampler, in.uv);
    return vec4<f32>(colour.rgb * colour.a, colour.a);
}
