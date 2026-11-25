Registers `command` under its own [`id`](Command::id) with the default
live [`CommandState`], returning the previously registered command with
that symbol, if any.

Re-registering a symbol replaces its descriptor **and** resets its state:
the new descriptor arrives with the default state, exactly as a first
registration would.
