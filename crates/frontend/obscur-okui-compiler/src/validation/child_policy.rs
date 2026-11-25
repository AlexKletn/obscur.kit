//! `ChildPolicy`: which children a component's contract admits (spec §2.13's
//! "allowed children", §2.12's "allowed child contracts").

#[doc = include_str!("../../docs/api/validation/child_policy.doc/ChildPolicy.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ChildPolicy {
    /// Any child is allowed: the contract does not restrict children.
    #[default]
    Any,
    /// No child is allowed — a leaf position, whose content the component
    /// synthesises itself (or which has none at all).
    None,
    /// Only children whose tag appears in this list are allowed. An empty list
    /// admits nothing (see the type docs): emptiness is never permissive here.
    Only(Vec<Box<str>>),
}

impl ChildPolicy {
    /// Returns whether a child element written as `tag` is admitted.
    #[must_use]
    pub fn admits(&self, tag: &str) -> bool {
        match self {
            Self::Any => true,
            Self::None => false,
            Self::Only(tags) => tags.iter().any(|allowed| allowed.as_ref() == tag),
        }
    }

    #[doc = include_str!("../../docs/api/validation/child_policy.doc/admits_content.md")]
    #[must_use]
    pub const fn admits_content(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns whether this policy forbids children outright.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[doc = include_str!("../../docs/api/validation/child_policy.doc/tags.md")]
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        match self {
            Self::Any | Self::None => &[],
            Self::Only(tags) => tags,
        }
    }

    #[doc = include_str!("../../docs/api/validation/child_policy.doc/allow.md")]
    pub fn allow(&mut self, tag: impl Into<Box<str>>) {
        match self {
            Self::Only(tags) => tags.push(tag.into()),
            Self::Any | Self::None => *self = Self::Only(vec![tag.into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_admits_every_tag_and_content() {
        let policy = ChildPolicy::Any;
        assert!(policy.admits("span"));
        assert!(policy.admits_content());
        assert!(policy.tags().is_empty());
    }

    #[test]
    fn none_admits_neither_a_tag_nor_content() {
        // The state the old `Vec` could not hold, and the whole point of the enum.
        let policy = ChildPolicy::None;
        assert!(!policy.admits("span"));
        assert!(!policy.admits_content());
        assert!(policy.is_none());
    }

    #[test]
    fn only_admits_the_listed_tags_and_leaves_content_alone() {
        let policy = ChildPolicy::Only(vec!["Tab".into()]);
        assert!(policy.admits("Tab"));
        assert!(!policy.admits("Panel"));
        assert!(
            policy.admits_content(),
            "a list of tags says nothing about a text run"
        );
    }

    #[test]
    fn an_empty_only_list_admits_nothing_rather_than_everything() {
        // Emptiness is never permissive in this type — the invariant that keeps
        // the old bug from coming back through a different door.
        assert!(!ChildPolicy::Only(Vec::new()).admits("span"));
    }

    #[test]
    fn allowing_a_tag_turns_restriction_on_from_either_other_state() {
        let mut from_any = ChildPolicy::Any;
        from_any.allow("Tab");
        assert_eq!(from_any, ChildPolicy::Only(vec!["Tab".into()]));

        let mut from_none = ChildPolicy::None;
        from_none.allow("Tab");
        assert_eq!(from_none, ChildPolicy::Only(vec!["Tab".into()]));

        from_none.allow("Panel");
        assert_eq!(
            from_none,
            ChildPolicy::Only(vec!["Tab".into(), "Panel".into()])
        );
    }
}
