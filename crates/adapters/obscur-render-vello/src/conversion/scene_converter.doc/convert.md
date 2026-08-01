Converts an obscur-scene [`DisplayList`] into a single [`vello::Scene`],
drawing any `filter: blur` subtree **inline and un-blurred**.

This is the back-compat entry point (used by callers and tests that do not
run the owned gaussian pass). For the real blur, the renderer uses
[`Self::convert_with_blurs`], which extracts blur subtrees into separate
sub-scenes. With no blur layer the two produce the identical scene.
