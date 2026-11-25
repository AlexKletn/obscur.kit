Whether the instance holds **content** that is not a member: a text child
or an `{expr}` interpolation.

Not a [`MemberUse`], because content has no tag — and the allowed-children
surface is a list of tags. It is recorded at all because
[`ChildPolicy::None`](crate::validation::ChildPolicy::None) is a claim about
the *position* rather than about names: `<Button>OK</Button>`, the case that
made "no children" worth expressing, has no element child to catch it by.
Collected through the same look-through as the members, so text inside an
`<If>` counts for the component the `<If>` splices into.
