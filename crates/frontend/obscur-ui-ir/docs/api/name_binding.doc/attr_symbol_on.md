Resolves an attribute name written **on `tag`** to the [`InternedSymbol`] it
binds to.

This is [`attr_symbol`] plus the one binding that depends on the tag the
attribute sits on: on [`OUTLET_TAG`], `name` is the *slot the outlet stands
for*, so it binds to the reserved [`SLOT`] symbol that `project_slots` reads —
exactly as a child's `slot="…"` does, because the two are the two halves of one
join and must be the same symbol or they never meet. On every other tag `name`
is an ordinary hashed prop, and `<slot class="x">` is an ordinary hashed prop
too: only the pair (`slot`, `name`) is special.

# The obstacle that was not one

The audit recorded that an outlet "cannot cheaply become" a tag because
"`attr_symbol` binds an attribute by its own spelling with no idea which tag it
sits on, so `name` cannot mean the slot's name on one tag and a hashed prop
elsewhere". The first clause is true of the *function*; the conclusion does not
follow. Of [`attr_symbol`]'s three production callers, the two that lower a
**document** — `UiIrViewBridge::lower_element` and `ViewCodegen::emit_element` —
hold `node.tag()` in the same scope in which they call it, and each is already
threading `TagPosition` out of that same node. The tag was not absent; it was
one argument away. That is a much cheaper finding than the one the audit wrote,
and the difference between the two is the whole cost of this feature.

The third caller is the one the audit's claim is true of, and it is why
[`attr_symbol`] is still here rather than replaced: `#[derive(Contract)]` hashes
a prop name out of a struct field, where there is no tag by construction. It
keeps the tagless function, which is exactly this one's non-outlet branch, so
the two cannot disagree.
