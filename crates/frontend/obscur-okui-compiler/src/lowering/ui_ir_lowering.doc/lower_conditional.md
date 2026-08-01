Lowers an `<If>`/`<ElseIf>`/`<Else>` into a
[`Conditional`](obscur_ui_ir::document::UiIrNodeKind::Conditional) node;
the branch condition (`cond={…}`) flows through as a property and the
guarded body as children (spec §2.11).
