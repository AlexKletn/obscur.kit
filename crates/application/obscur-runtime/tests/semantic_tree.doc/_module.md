The semantic tree (spec §7.1): what the runtime derives from the retained tree,
and how it reaches the platform.

The derivation comes first — a `button` element with a text child becomes a
`Button` node named "Save" over a `Text` node named the same, and the same live
tree yields an *equal* snapshot twice, because an assistive adapter that
re-converts a tree has to get the same answer.

The rest is publication, and its bridge deliberately **keeps** what it is handed
rather than counting calls: a test that only counted pushes would pass while the
runtime published an empty or stale tree. So the assertions read what is *in*
the snapshot the bridge received — the button's name, and the disabled state
that §7-A fixed in the builder.

Three properties of the push follow. An **ordinary** frame publishes, with
nobody calling `push_accessibility` — that call is the manual path, and it has
its own test because it is still part of the API. **Installing** a bridge
publishes what is already on screen, so a host that attaches a screen reader
late does not have to wait for an unrelated change. And a frame that changed
neither the tree nor the focus publishes **nothing**: the semantic tree is a
full walk of the retained tree, so re-publishing an idempotent frame would put
that walk on every frame forever.

The fourth property — that a focus move publishes both the move and a tree that
agrees about it — is asserted in `ui_runtime.rs`, where the test can read the
focus field the record is minted from.
