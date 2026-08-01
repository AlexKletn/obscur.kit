Carries a screen front-end failure into the one error type a windowed
application sees, keeping the parse/compile split (and the diagnostics)
intact — the `.okui` front end is display-free and has its own error type, so
this is the only place the two meet.
