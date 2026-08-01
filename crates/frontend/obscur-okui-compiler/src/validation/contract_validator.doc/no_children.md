Declares that the component takes **no children at all** (fluent) — the
[`ChildPolicy::None`] leaf state.

This is the §2.13 "allowed children" case the contract could not express:
an empty allowed-children list already meant *any* child, so a component
whose content is synthesised from its own model — every shipped widget
except `Dialog` — had no way to refuse the children an author wrote, and
`<Button>OK</Button>` compiled into a text run that nothing ever mounts.

It forbids element children *and* text/`{expr}` content: the claim is that
the position is a leaf, and text renders nowhere in a leaf exactly as an
element does.
