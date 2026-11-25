[`UiRuntime`]'s styling surface: what goes into the cascade and what comes out of
it.

The inputs are the three stylesheet sources (the reactive author set, the UA
default, and the question of whether one is installed), the host-driven pseudo
states, and the locale — a language is a cascade *and* shaping input, which is
why it is read here rather than beside the text ports that set it. The output is
one reader, `computed_style`. Resolving the cascade itself is the style phase's,
and it lives with the frame.
