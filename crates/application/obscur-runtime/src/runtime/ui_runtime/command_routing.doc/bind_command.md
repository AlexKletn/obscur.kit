Binds `shortcut` to the app-owned command symbol `command` in the
runtime's command router, so a matching key press at
[`handle_key`](Self::handle_key) resolves to — and dispatches — that
command (spec §6.7).

`command` is the **same** [`InternedSymbol`] the application registers a
handler for via [`on_command`](Self::on_command) (and may also bind an
`on-click`/`on-change` event to in the View): keyboard and pointer input
share one command identity, so a bound shortcut runs the same handler a
click would. Registration order is priority: an earlier binding shadows a
later one with the same shortcut (see [`CommandRouter::bind`]). Binding
does not itself schedule a frame.
