[`UiRuntime`]'s accessibility surface (spec §7.1): the semantic projection of the
retained tree in both of the two shapes a consumer asks for, the bridge a host
installs, and the three ways that bridge gets told.

Projecting is `SemanticTreeBuilder`'s and remembering what the port has already
been told is `AccessibilityPublisher`'s. What is here is the join the two need
and neither may hold: a push reads the runtime's focus and the snapshot the
runtime alone can build, then hands both to the publisher.

The three pushes are one method each because they answer three different
questions. `push_accessibility` is the host's — it names the bridge, so it fires
unconditionally. `publish_accessibility` is the frame's, and it fires only when
the publisher says the tree or the focus has actually moved; that gate is why
a steady frame costs nothing here. `publish_accessibility_now` is the pair's
shared tail, and the reason installing a bridge publishes immediately rather
than at the next frame: a screen reader that attaches to a running app must not
have to wait for the user to do something before it can read the screen.
