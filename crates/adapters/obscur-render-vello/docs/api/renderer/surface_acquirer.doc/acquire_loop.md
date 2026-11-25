The thread body: wait for a request, serve it, repeat.

A lost surface is reconfigured and the next request tries again. There is
nothing here to report it to — the renderer owns the diagnostics and is on the
other thread — which is a real limit of putting the acquire here and is worth
knowing rather than hiding.
