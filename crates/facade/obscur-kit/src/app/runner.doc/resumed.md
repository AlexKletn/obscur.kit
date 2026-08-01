Creates the window + GPU renderer once the platform is ready, wires the text
stack, enables IME, and schedules the first frame. A window or GPU failure
records the reason and exits the loop (never panics — the headless CI path).
