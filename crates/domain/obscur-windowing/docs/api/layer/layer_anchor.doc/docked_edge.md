The edge this anchor docks to, if it describes a dock at all.

Two shapes count as a dock, and they are the two that occur in practice:

- **one edge**, which docks to that edge;
- **three edges**, which is how a bar is spelled — a top bar is anchored top,
  left and right so that it stretches across, and it docks to the edge
  *opposite the one it left out*.

Everything else answers [`None`]. Two opposite edges describe a band with no
edge to call its own; two adjacent edges describe a corner; all four describe
a surface covering the screen; none describes a floating surface the platform
places. None of those is a dock, and a backend that can only dock has to be
told so rather than left to guess.
