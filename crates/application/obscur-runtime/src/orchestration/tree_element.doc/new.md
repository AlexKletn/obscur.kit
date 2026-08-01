Creates a **stateless** cursor over `id` within `tree`.

The cursor carries no interaction-state source, so
[`pseudo_state`](MatchElement::pseudo_state) always reports
[`PseudoState::empty`]. This is the path `run_matched` uses, where no
live interaction state exists. A [`UiRuntime`](crate::UiRuntime) — and so
the headless host, which drives one — takes
[`with_pseudo`](Self::with_pseudo) instead.
