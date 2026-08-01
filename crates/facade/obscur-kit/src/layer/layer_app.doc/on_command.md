Registers the handler for a command bound in the view or the sheet.

Takes either §2.3 shape, as [`App::on_command`](crate::app::App::on_command)
does — `|| …` when the handler needs nothing, `|e: &mut UiEvent| …` when it
wants the event's `target`, §2.4's pointer/key payload, the trust flag or the
`stop_propagation` / `prevent_default` steering. A panel is a §1.4 standalone
entry point like `App`, so it must not be the one that still hands a handler
nothing.

Positional registration (§2.6's links 1–3) and §4.12 stylesheet invalidation
are **not** wired here yet: [`AppControl`](crate::control::AppControl) is
display-free and would work, but this runner does not apply it at its frame
boundary. A panel therefore still has the one flat command namespace.
