Opens `scope` as a focus-containment scope (spec §7.1), pushing it as the
innermost open scope.

While open, the scope forms link 2 of §2.6's routing chain for any
command originating at one of its [`members`](FocusScope::members): its
[`on_scope_command`](Self::on_scope_command) handlers are offered the
command after the focused node itself but *before* any ancestor. Scopes
nest — a menu opened from a dialog pushes a second scope — and the
innermost is consulted first.

Opening the same [`FocusScopeId`] twice replaces the earlier scope's
membership rather than stacking a duplicate, so a scope whose members
changed across a structural frame is re-opened with the new node ids.
The scope's [`traps`](FocusScope::traps) flag is *not* consulted by
command routing: trapping governs tab navigation, and a scope routes
commands whether or not it traps.
