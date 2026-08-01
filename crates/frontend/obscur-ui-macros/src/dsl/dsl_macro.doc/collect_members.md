Appends `element`'s members, looking through the control flow that
splices into it — the same walk the file front end makes, because the
emitter makes no distinction between the two front ends' output.

The same walk records [`ElementUse::content`]: a `"text"` child or a
`{expr}` interpolation, which is not a member because it has no tag to be
checked against an allowed-children list. A contract declaring *no*
children refuses it all the same — that is what makes
`okui! { <Button label="x">"OK"</Button> }` a compile error rather than a
text run nothing mounts.
