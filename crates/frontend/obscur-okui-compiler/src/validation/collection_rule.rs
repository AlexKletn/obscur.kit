#![doc = include_str!("../../docs/api/validation/collection_rule.doc/_module.md")]

#[doc = include_str!("../../docs/api/validation/collection_rule.doc/check_collection.md")]
pub fn check_collection(_tag: &str) -> Result<(), &'static str> {
    Ok(())
}

/// Diagnostic for a repeated structural key on the same authored node.
pub const DUPLICATE_KEY: &str = "only one `key` attribute is allowed";

/// Diagnostic for a structural key written without a value.
pub const KEY_REQUIRES_VALUE: &str = "`key` must have an explicit value";

/// Reports repeated direct collection slots at their authored anchors.
pub fn duplicate_collection_slots<T: AsRef<str>, A>(
    children: impl IntoIterator<Item = (T, A)>,
) -> impl Iterator<Item = (A, &'static str)> {
    let mut seen = [false; 3];
    children.into_iter().filter_map(move |(tag, anchor)| {
        let (index, message) = match tag.as_ref() {
            "Empty" => (0, "a collection accepts only one `<Empty>` slot"),
            "Loading" => (1, "a collection accepts only one `<Loading>` slot"),
            "Separator" => (2, "a collection accepts only one `<Separator>` slot"),
            _ => return None,
        };
        std::mem::replace(&mut seen[index], true).then_some((anchor, message))
    })
}

#[cfg(test)]
mod tests {
    use super::check_collection;

    #[test]
    fn plain_for_is_accepted() {
        assert_eq!(check_collection("For"), Ok(()));
    }

    #[test]
    fn virtual_for_is_now_accepted() {
        // §2.11's `<VirtualFor>` is a first-class construct: it lowers to a
        // virtual-collection marker the runtime realizes into a viewport-bounded
        // window (spec §4.9), so the rule no longer refuses it.
        assert_eq!(check_collection("VirtualFor"), Ok(()));
    }

    #[test]
    fn an_ordinary_tag_is_not_this_rules_business() {
        for tag in ["div", "Button", "If", "Else", "Empty", "Separator"] {
            assert_eq!(check_collection(tag), Ok(()), "{tag}");
        }
    }
}
