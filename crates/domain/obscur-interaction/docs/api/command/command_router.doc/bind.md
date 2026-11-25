Binds `shortcut` to `command`, appending it at the lowest priority.

Duplicate shortcuts are allowed; the earliest-registered one wins during
[`resolve`](CommandRouter::resolve).
