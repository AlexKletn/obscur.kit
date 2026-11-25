What one dispatch did, so the caller can decide what it owes.

Two facts, not one, because they answer different questions and disagree in
both directions: a listener can run on an event no link ever claims, and a
claimed command whose handler mutated nothing still needs the frame. The
**claim** is what the dispatch *returns* to an application; the **run** is why
a frame is owed, since either a listener or a handler may have mutated
application state.
