Parses the optional leading prelude of a `radial-gradient`: an ending shape,
an ending size, and an `at <position>` center, in any order (`at` last).
Returns [`None`] when the part contains no prelude token (so it is treated as
the first color stop instead) **or** when it contains an unrecognised token (a
malformed prelude, which drops the whole gradient once stop parsing then also
fails).
