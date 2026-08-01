End-to-end proof that the two historical `.okui` "bridge gaps" — real widgets
and `<img src>` — are closed, driven through the *real* frontend
(parse → compile → lower) rather than a hand-built IR node.

Both `.okui` lowering paths resolve names through the shared
`obscur_ui_ir::name_binding` registry, so this file checks each gap on both:

- the **runtime bridge** path renders an authored `<img src>` all the way to
  a `PaintCommand::DrawImage` (registry → reserved `SRC` → decoded pixels),
  and binds `<Button>` to the real `Button::COMPONENT_ID`; and
- the **codegen** path emits, from the same compiled IR, Rust source carrying
  those same real handles.
