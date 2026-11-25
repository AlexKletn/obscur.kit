One out-of-workspace fixture and what is expected of it.

A profile is a fixture plus the answer to "what would a defect here look like?".
For the windowed one that is a build failure and nothing else. For the layer one
it is a build failure **or** a warning in code this profile is the only compiler
of — see [`watched`](Self::watched).
