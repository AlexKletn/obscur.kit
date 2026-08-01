§2.6's five-link chain: focused node, focus scope, ancestors, window,
application fallback.

Four tests about what makes this a *chain* rather than a set of registries: each
link is consulted only when every inner link declined, and the first claim stops
the walk. Two of them take one link each — the ancestor, reached because the
focused node registered nothing, and the application fallback, reached because
nothing else claimed the symbol at all — with every earlier link asserted empty
so the test cannot pass for the wrong reason.

The third is the one that makes links 4 and 5 two links rather than one map
consulted twice. A `UiRuntime` is one window surface, so a handler registered on
it is that window's alone; an `AppCommands` registered **once** and installed on
two runtimes serves both, and a runtime holding a separate registry sees neither.
The fourth is the degenerate route: with nothing focused the positional links
have no anchor, so a route is the two non-positional links — and never empty.

**Two tests of this theme stayed in `ui_runtime.rs`**, both because they reach
past the public API and this campaign does not widen an API to relocate a test.
`the_five_links_are_tried_in_the_specs_order_and_the_first_claim_stops_the_chain`
takes the window link away by calling `routing.remove_command`, a private field;
`a_focus_scope_link_appears_only_for_its_own_members` dispatches from a node that
is not the focused one through the private `dispatch_command_from`. Both are the
chain read from the inside, which is exactly what a unit test is for.
