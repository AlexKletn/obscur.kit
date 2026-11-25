/// One attribute written on an element instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeUse<'a> {
    /// The attribute's name — for an event, the local name after `on:`.
    pub name: &'a str,
    /// Whether this is an `on:<event>` binding rather than a prop.
    pub event: bool,
}
