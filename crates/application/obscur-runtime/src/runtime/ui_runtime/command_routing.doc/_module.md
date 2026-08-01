[`UiRuntime`]'s command surface (spec §2.6, §6.7): the registry and router a host
declares its commands in, the shortcuts bound to them, the handlers installed at
every level of the routing chain — node, focus scope, application — the two
questions a caller can ask about a route, and the dispatch that actually sends
one down it.

The declarative half is all one entity's, `HandlerRegistry`'s, which is why the
focus *scopes* are here and the focused *node* is not: a scope is a link in the
routing chain, opened and closed on the registry, while the focus itself is a
runtime field that the chain merely starts from. `command_route` answers where a
command would go without sending it.

The dispatch half is the other side of the same subject, and it is here because
every one of its callers is somewhere else. `dispatch_command_event` is the one
place §2.3's event propagation and §2.6's routing chain meet — the listener walk
is steps 1–3, the chain is step 4 — and everything above it is a different way
of arriving at it: `dispatch_command` from a host call, `dispatch_command_from`
from a programmatic dispatch with no device behind it, `activate` from a pointer
click or an Enter/Space press, and `dispatch_text_input` / `dispatch_change`
from a field's own edits. Keeping them together is what stops a second dispatch
path being invented beside the first: each of them differs only in the envelope
it builds and the binding it resolves, never in what dispatch means.
