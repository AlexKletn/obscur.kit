use super::{CompositeContent, Draw};

pub(super) enum GroupInput<'a> {
    Draw(&'a Draw),
    Composite(CompositeContent),
}
