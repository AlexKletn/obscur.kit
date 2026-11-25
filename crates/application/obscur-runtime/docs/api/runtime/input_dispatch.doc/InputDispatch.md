The result of dispatching one queued
[`InputEvent`](super::InputEvent), in the order the events were queued
(spec §3.4 phase 3).

Each variant carries exactly what the matching immediate entry point returns —
[`handle_pointer`](super::UiRuntime::handle_pointer) and friends — because the
queue dispatches through those same functions. Queuing an event and handling it
immediately differ in *when*, never in *what*.
