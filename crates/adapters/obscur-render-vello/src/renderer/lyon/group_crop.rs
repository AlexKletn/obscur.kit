use super::target_extent::rounded_target_extent;

use super::mesh::{Draw, Operation};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct GroupCrop {
    pub(super) origin: [u32; 2],
    pub(super) extent: (u32, u32),
}

impl GroupCrop {
    pub(super) fn for_draws(draws: &[Draw], extent: (u32, u32)) -> Option<Self> {
        let mut groups = draws.iter().filter_map(|draw| match draw.operation {
            Operation::BeginGroup | Operation::EndGroup { .. } | Operation::Backdrop { .. } => {
                Some(draw.operation)
            }
            _ => None,
        });
        if groups.next()? != Operation::BeginGroup {
            return None;
        }
        let Operation::EndGroup {
            bounds: [x, y, width, height],
            blend: None,
            blur_radius: 0.0,
            color_filter: None,
            stencil_depth: 0,
            encode_srgb: false,
            ..
        } = groups.next()?
        else {
            return None;
        };
        if groups.next().is_some() {
            return None;
        }
        Self::from_bounds([x, y, width, height], extent)
    }

    pub(super) fn from_bounds([x, y, width, height]: [u32; 4], extent: (u32, u32)) -> Option<Self> {
        if x >= 16_384 || y >= 16_384 || width == 0 || height == 0 {
            return None;
        }
        let w = rounded_target_extent(width, extent.0);
        let h = rounded_target_extent(height, extent.1);
        let full_area = u128::from(extent.0) * u128::from(extent.1);
        (u128::from(w) * u128::from(h) * 4 < full_area * 3).then_some(Self {
            origin: [x, y],
            extent: (w, h),
        })
    }

    pub(super) fn vertex_start(&self) -> u32 {
        // Modes 0 and 1 select the ordinary composite shaders.
        (2 + self.origin[0] + (self.origin[1] << 14)) * 3
    }
}
