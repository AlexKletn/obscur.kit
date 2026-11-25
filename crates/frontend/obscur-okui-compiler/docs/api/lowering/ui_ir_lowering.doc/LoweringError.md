Why lowering a `.okui` document into UI IR failed, or a non-fatal lowering
diagnostic (spec §2.15).

[`EmptyDocument`](Self::EmptyDocument) and [`MissingTagName`](Self::MissingTagName)
are *fatal*: they prevent building a tree and surface as the `Err` of
[`UiIrLowering::lower`](super::UiIrLowering::lower). [`MissingLoopKey`](Self::MissingLoopKey) is *non-fatal*:
the `<For>` is still lowered (keyless), but the missing mandatory key is
reported through the lowering's accumulated diagnostics (spec §2.11).
