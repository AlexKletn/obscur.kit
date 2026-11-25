#[derive(Clone, Copy, PartialEq)]
pub(super) enum Operation {
    Paint,
    Image(usize),
    Push,
    Pop,
    BeginGroup,
    Backdrop {
        sample_bounds: [u32; 4],
        radius: f32,
        stencil_depth: u32,
    },
    EndGroup {
        encode_srgb: bool,
        blend: Option<u32>,
        opacity: f32,
        bounds: [u32; 4],
        color_filter: Option<u32>,
        blur_radius: f32,
        stencil_depth: u32,
    },
}
