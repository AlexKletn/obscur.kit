use super::{CompositeContent, DrawContent};

#[derive(PartialEq)]
pub(super) enum ContentOperation<'a> {
    Draw(DrawContent<'a>),
    Composite(CompositeContent),
}
