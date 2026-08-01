Replaces the root display list and **advances the generation**.

Replacing the root is a content change, so the generation moves to
[`SceneGeneration::next`]; any [`SceneSnapshot`] taken before this call
now reports stale against the new generation.
