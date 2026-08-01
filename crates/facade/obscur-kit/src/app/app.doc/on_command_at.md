Registers `handler` for `command` **at the node** `target` names — links 1 and
3 of §2.6's routing chain, where the same symbol runs different logic
depending on where the command came from.

`target` is a §7.5 stable identifier: write `test-id="save"` on the element
(in `.okui` markup, or as a `TEST_ID` property in a Rust view) and name it
here with [`SelectorTarget::test_id`]. A [`NodeId`](crate::identity::NodeId)
works too, for a node an application already read back from the tree — but a
name is what an application configuring itself before `run` can have, since no
node id exists until a frame has built the tree.

The name is resolved after the first commit, and resolved again whenever the
element it was bound to stops carrying it — so the registration follows the
**name**, across a rebuilt subtree. A name that matches nothing stays unbound and
never fires; ask [`AppControl::bound_node`] instead of guessing.

A handler here **claims** the command and stops the chain: no ancestor, no
window handler ([`on_command`](Self::on_command)) and no application fallback
sees it. Both handler shapes are accepted, as in `on_command`.
