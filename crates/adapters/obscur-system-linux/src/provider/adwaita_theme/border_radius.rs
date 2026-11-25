pub(super) enum BorderRadius {
    Pixels(f32),
    WindowVariable,
}

impl BorderRadius {
    pub(super) fn parse(value: &str) -> Option<Self> {
        if value.trim() == "var(--window-radius)" {
            Some(Self::WindowVariable)
        } else {
            pixels(value).map(Self::Pixels)
        }
    }
}

pub(super) fn pixels(value: &str) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    let number: f32 = value.strip_suffix("px")?.parse().ok()?;
    (number.is_finite() && number >= 0.0).then_some(number)
}
