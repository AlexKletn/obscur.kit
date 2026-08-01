§6.16 "- edge/reserved gesture regions;" / "- gesture arena;" /
"System gestures have priority by default."

A horizontal drag starting at x=8, inside a 20px reserved system-back strip:
the system gesture wins the arena and the application's pan recognizer does
not become the winner. Neither recognizer is given an explicit priority —
system priority is the *default*, not something the caller opts into.
