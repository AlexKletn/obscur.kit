A function call such as `calc(…)`, `rgb(…)`, `var(…)`, or `translate(…)`.

A bare parenthesised group `( … )` is represented as a function with an
empty [`name`](ParsedValue::Function::name). Arguments are the raw nested
component list, commas and operators included.
