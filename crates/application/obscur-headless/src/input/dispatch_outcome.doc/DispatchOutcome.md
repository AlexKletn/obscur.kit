What a dispatched [`SyntheticInput`](crate::input::SyntheticInput) actually
did (spec §7.4 "typed dispatch results").

Every field is observed from the *real* input path — the same
[`UiRuntime`](obscur_runtime::UiRuntime) call a windowed host makes — so a
headless assertion cannot pass while a real click would fail.

# What is and is not here

The spec lists target, propagation path, default action, focus changes, state
changes and layout changes. This carries what the runtime genuinely reports
today: the resolved [`target`](Self::target), the
[`propagation_path`](Self::propagation_path) (target → root, the chain a
command binding is looked up along), the dispatched
[`command`](Self::command) — the default action of a completed click or a
bound key — and whether [`hover`](Self::hover_changed) or
[`focus`](Self::focus_changed) moved.

Explicit state-change and layout-change records are **not** included, because
the runtime does not produce them: reporting them would mean inventing a
second, unverified source of truth. A test asserts those by inspecting the
tree and layout boxes it already has.
