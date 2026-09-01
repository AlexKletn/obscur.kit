The message for a method invoked on a value reached through a field.

It names the refused call and states the rule as the check can honestly state
it: the shape is refused because nothing syntactic separates `row.keys.len()`
from `services.http.get(url)`, not because a `.len()` on a `Vec` *is* a
service. Naming the wrong rule sends an author looking for a service they never
wrote.
