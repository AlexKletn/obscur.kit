/// Measured content-box axes of a query container. An absent axis cannot be queried.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryContainer {
    /// Optional authored name.
    pub name: Option<Box<str>>,
    /// Queryable content width in logical pixels.
    pub width: Option<f32>,
    /// Queryable content height in logical pixels.
    pub height: Option<f32>,
    /// Whether the inline axis is vertical.
    pub vertical: bool,
}

impl QueryContainer {
    pub(super) fn dimensions(&self) -> [Option<f32>; 4] {
        let (inline, block) = if self.vertical {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        [self.width, self.height, inline, block]
    }
}
