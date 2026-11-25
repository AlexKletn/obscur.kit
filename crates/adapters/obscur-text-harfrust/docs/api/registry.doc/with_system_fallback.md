Attaches the host lookup consulted when **no** registered family covers a
run (spec §5.10).

Without one, a character outside the registered faces' `cmap`s resolves
to nothing and is simply not drawn — a blank where a glyph should be.
With one, the first appearance of such a character costs one host lookup;
the face is then kept and answers instantly thereafter.

Attach it **after** every `register` call: discovered faces take ids
above the registered ones, and a `register` afterwards would collide.
