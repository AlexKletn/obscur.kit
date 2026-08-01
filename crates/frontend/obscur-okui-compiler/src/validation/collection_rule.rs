#![doc = include_str!("collection_rule.doc/_module.md")]

#[doc = include_str!("collection_rule.doc/check_collection.md")]
pub fn check_collection(_tag: &str) -> Result<(), &'static str> {
    Ok(())
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
