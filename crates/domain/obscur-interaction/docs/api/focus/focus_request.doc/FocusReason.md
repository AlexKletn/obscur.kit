Why focus is being moved (spec §7.1).

A tightly-coupled companion to [`FocusRequest`](super::FocusRequest): the request names *what* to
focus while the reason records *why*, letting the runtime distinguish, for
example, keyboard-driven from pointer-driven focus. It is passed alongside a
request to the focus state machine rather than embedded in the request enum.
