§6.5 Fullscreen and presentation: "- restore previous state;"

A presentation request the backend cannot service is refused as a typed
result and leaves the window's state untouched.

WEAKENED 2026-07-19: the assertion that fired was
`entered.is_success()`, whose message read "§6.5: borderless fullscreen is
the emulatable form". That sentence is not in the spec. §6.5 reads, in full:
"Modes may include:" / "- normal;" / "- borderless;" / "- exclusive where
available;" / "- kiosk;" / "- presentation." — *may* include — followed by
the optional controls (among them "- restore previous state;") and "The
application owns policy." Nothing designates borderless as the form a
compositor-less backend must emulate, and §6.2's four capability categories
make `unsupported` a legitimate answer from `NullWindowBackend`. The
restore-previous-state assertions went with it, since a backend that never
entered fullscreen cannot be asked what it restored on leaving.

What replaces them is the §6.2 rule the sibling case at the top of this file
also encodes, and it still discriminates: a refusal must not have applied the
property anyway. The same requirement IS separately tested against the one
backend that implements presentation — see P6-06b in the real-gap register,
where `WinitWindowBackend::set_presentation` silently drops
`restore_previous_state` while reporting `Performed(Native)`.

Candidate spec amendment: say which presentation modes a backend must support
or emulate to claim §6.5 at all, and what "- restore previous state;"
restores (position, size, maximized flag, monitor).
