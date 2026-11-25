/// One authored `<slot …/>`, described in the shape both front ends can build
/// from their own syntax (spec §2.12).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutletUse<'a> {
    /// Every attribute name on the tag *except* `name`, in authored order —
    /// including event bindings, spelled as the author wrote the event
    /// (`on:click` → `"on:click"`), so the message quotes them back.
    pub extra_attributes: Vec<&'a str>,
    /// Whether the tag has any children (elements, text or interpolations).
    pub has_children: bool,
}
