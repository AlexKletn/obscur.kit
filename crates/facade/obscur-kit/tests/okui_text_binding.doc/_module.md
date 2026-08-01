End-to-end: a body `{expr}` **renders its value**, on both front ends
(spec §2.8, §2.10).

`<div>{user.name}</div>` rendered the literal text `user.name` — the nine
characters of the binding's own source. Not a missing feature: a document that
silently showed its own spelling. The attribute case was fixed and this one,
the more common one, was left live, and the two front ends drifted while it
was: the macro emitted the path as text, and the `.okui` file emitted an
element named `Expression` that renders as an empty box.

One representation now serves both — `UiIrNode::interpolation`, which carries
the expression rather than its spelling — so `ViewCodegen` emits the same text
node from either. The macro tests below check a `View` built by the real
`okui!` with real locals in scope; the file test checks the Rust the file front
end emits for the same document.
