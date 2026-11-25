#[derive(Clone, Copy, PartialEq)]
pub(crate) struct RoundedRegion {
    size: [f32; 2],
    radii: [[f32; 2]; 4],
}

impl RoundedRegion {
    pub(crate) fn new(size: [f32; 2], radii: [[f32; 2]; 4]) -> Option<Self> {
        (size
            .into_iter()
            .all(|length| length.is_finite() && length > 0.0)
            && radii.iter().any(|&[x, y]| x > 0.0 && y > 0.0))
        .then_some(Self { size, radii })
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub(crate) fn rectangles(self, mut add: impl FnMut(i32, i32, i32, i32)) {
        let [width, height] = self.size;
        let [top_left, top_right, bottom_right, bottom_left] = self.radii;
        let rows = height.floor() as i32;
        let mut run_start = 0;
        let mut run = (0, 0);
        for y in 0..rows {
            let top = y as f32;
            let bottom = height - (y + 1) as f32;
            let left = inset(top_left, top).max(inset(bottom_left, bottom));
            let right = inset(top_right, top).max(inset(bottom_right, bottom));
            let start = left.ceil() as i32;
            let end = (width - right).floor() as i32;
            let span = (start, end);
            if span != run {
                if run.1 > run.0 && y > run_start {
                    add(run.0, run_start, run.1 - run.0, y - run_start);
                }
                run_start = y;
                run = span;
            }
        }
        if run.1 > run.0 && rows > run_start {
            add(run.0, run_start, run.1 - run.0, rows - run_start);
        }
    }
}

fn inset([x, y]: [f32; 2], distance: f32) -> f32 {
    if x <= 0.0 || y <= 0.0 || distance >= y {
        return 0.0;
    }
    let t = 1.0 - distance / y;
    x * (1.0 - (1.0 - t * t).sqrt())
}

#[cfg(test)]
#[path = "rounded_region.tests.rs"]
mod tests;
