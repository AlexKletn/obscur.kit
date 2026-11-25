Lowers a single element (recursively) into a [`UiIrNode`].

The element-shaped `<For>`/`<VirtualFor>` and `<If>`/`<ElseIf>`/`<Else>`
control-flow constructs (spec §2.11) lower to the dedicated
[`Loop`](obscur_ui_ir::document::UiIrNodeKind::Loop) and
[`Conditional`](obscur_ui_ir::document::UiIrNodeKind::Conditional) IR
kinds; every other element lowers to a component or element node.
