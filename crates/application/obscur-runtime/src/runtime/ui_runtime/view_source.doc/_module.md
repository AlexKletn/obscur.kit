[`UiRuntime`]'s two sources of a desired view: the retained root builder the
reactive path rebuilds from, and the component registry a `View` names its
components out of.

They are one subject because a frame needs both to produce anything: the builder
says what the tree should be, and the registry is what its component nodes
resolve through. Running either is the reconcile phase's.
