Formats the pattern with `args`, selecting plural arms under `rules`.

`rules` is the *formatting locale's*, which is the whole point: the same
pattern text formats differently under `ru` and `en` because the category
of 21 differs. This method uses each argument's original spelling.
[`Localizer::format`](crate::catalog::Localizer::format) also applies the
active locale's number formatting to interpolated numeric arguments.

# Missing and mismatched arguments

Formatting never fails and never panics — it is on the frame path, and a
missing argument is a catalogue bug that must not take the UI down.
Instead it renders visibly:

- an unbound `{name}` renders as `{name}`, brace-and-all, so it is obvious
  on screen and greppable in a screenshot;
- a `plural` over an unbound or non-numeric argument takes the `other`
  arm, which is the only arm guaranteed to exist;
- `#` outside any number renders empty.
