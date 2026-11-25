Builds a snapshot directly from a display list at a given generation.

Takes ownership of the list; [`SceneStatistics`] are not computed here — see
[`SceneSnapshot::statistics`], which counts them when someone asks.
