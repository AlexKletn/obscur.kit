End-to-end proof that the **widget library is clickable**: a real
[`Button`](obscur_kit::widgets::button::Button), rendered through the
embeddable [`UiRuntime`](obscur_kit::runtime::UiRuntime), dispatches its bound
command when the runtime detects a pointer click on it — and the reactive
re-render loop rebuilds the view from the mutated state.

This exercises the fix that aligns the `Button`'s click binding to the
reserved [`EVENT_CLICK`](obscur_kit::view::view::EVENT_CLICK) event symbol the
runtime's click resolver looks up. Before the fix the button bound a
widget-local event symbol, so a runtime click on it resolved to no command.
