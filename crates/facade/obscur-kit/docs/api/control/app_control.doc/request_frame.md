Asks for a frame, for a state change the runtime cannot have observed.

Not needed after a command handler — the runtime schedules a frame for the
input that ran it — but needed when application state changed for a reason the
runtime never saw: a timer, a background task landing, a message from another
thread's channel.
