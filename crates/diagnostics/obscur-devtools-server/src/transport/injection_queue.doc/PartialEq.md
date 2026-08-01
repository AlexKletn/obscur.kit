Two sinks are equal when they push into the same queue, so a
[`DevToolsEndpoint`](crate::endpoint::DevToolsEndpoint) keeps its derived
equality. A `Weak` has no value to compare; identity is the only thing a
channel end *has*.
