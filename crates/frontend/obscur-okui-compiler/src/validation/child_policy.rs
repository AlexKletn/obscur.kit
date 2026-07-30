//! `ChildPolicy`: which children a component's contract admits (spec §2.13's
//! "allowed children", §2.12's "allowed child contracts").

/// Which children a [`ComponentContract`](super::ComponentContract) admits,
/// named by tag.
///
/// # Why this is an enum and not a list
///
/// It used to be one field — `allowed_children: Vec<Box<str>>` — whose
/// *emptiness* meant "any child". That gave the contract model two states where
/// §2.13 needs three, and the missing one was the state almost every shipped
/// component wanted: `Button`, `Input`, `Checkbox`, `Radio`, `Select` and the
/// collection widgets synthesise their content inside their own `view()` from
/// their own model, and `Image`/`Canvas`/`ForeignSurface` are leaves. So
/// `<Button>OK</Button>` authored a text run that nothing mounts and nothing
/// renders, and it compiled clean — because the only way to spell "no children"
/// was the very same empty list that already meant "any".
///
/// Overloading emptiness a second time (an empty list meaning *none* in some
/// contexts) would have made the field's meaning depend on who was reading it.
/// Naming the three states removes the question: **emptiness never means
/// "unrestricted" anywhere in this type**, and [`Only`](Self::Only) with an empty
/// list therefore admits nothing, exactly as [`None`](Self::None) does.
///
/// # The mirror
///
/// The three cases are `obscur_view::contract::ChildrenSurface`'s and
/// `obscur_view::contract::ChildContract`'s, deliberately spelled the same:
/// `ChildContract` decides membership by
/// [`ComponentId`](obscur_identity::ComponentId), `ChildrenSurface` by tag name
/// for tooling, and this by tag name for the `.okui` front ends. The bridge in
/// [`surface_bridge`](super) crosses `ChildrenSurface` into this, and it used to
/// have to flatten `None` into `Any` because this type could not hold it — the
/// loss the enum removes.
///
/// # Content is not a tag
///
/// A text child or an `{expr}` interpolation has no tag, so a *list* of allowed
/// tags cannot speak about it: [`Only`](Self::Only) restricts which element tags
/// may appear and leaves content alone, which is what "allowed children" means as
/// a list of names. [`None`](Self::None) is a claim of a different kind — that the
/// position is a leaf — and content violates it exactly as an element does. See
/// [`admits_content`](Self::admits_content).
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

    /// Returns whether non-element **content** — a text child or an `{expr}`
    /// interpolation — is admitted.
    ///
    /// Only [`None`](Self::None) refuses it, and for a reason that is not about
    /// names: it says the position takes no children at all, so text written
    /// there renders nowhere. [`Only`](Self::Only) is a list of *tags*, and a text
    /// run has none to be absent from — restricting content on the strength of a
    /// tag list would be a rule the list does not state.
    #[must_use]
    pub const fn admits_content(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns whether this policy forbids children outright.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns the explicitly allowed child tags — the [`Only`](Self::Only) list,
    /// or an empty slice for the two policies that name no tags.
    ///
    /// An empty slice is therefore *not* a permissive answer: ask
    /// [`admits`](Self::admits) or match the policy, never
    /// `tags().is_empty()`.
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        match self {
            Self::Any | Self::None => &[],
            Self::Only(tags) => tags,
        }
    }

    /// Adds `tag` to the allowed list, switching to [`Only`](Self::Only) from
    /// either other policy.
    ///
    /// Naming an allowed child is how a contract *turns on* child restriction, so
    /// this necessarily leaves [`Any`](Self::Any); it also leaves
    /// [`None`](Self::None), because "no children, except this one" is a
    /// contradiction and the later, more specific statement is the author's
    /// intent.
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
