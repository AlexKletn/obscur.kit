The diagnostic statistics of the list this snapshot holds, counted on the spot.

Not stored. A snapshot is taken at least once per frame — twice, on the layer
runner's path — and this used to be computed eagerly in the constructor, so an
otherwise refcount-cheap snapshot carried a full walk of every command in the
frame each time. Nothing on the frame path reads it: the only caller outside
tests is the headless render-graph summary, and that one counts the scene
directly. Counting here on demand costs the same walk to whoever actually wants
the numbers, and nothing to everyone else.

Not a cache either, and nothing to invalidate: the list a snapshot holds is
frozen at the generation it was taken (`set_root` replaces the handle rather
than mutating in place), so the answer is a pure function of it.
