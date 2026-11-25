The hyphenation port used to break an over-wide word at a valid point
(spec §5.10), or [`None`] when no hyphenator is installed.

The default is [`None`]: text wraps exactly as it did before hyphenation
existed (no hyphen glyph is even shaped), which is what the cross-backend
determinism boundary relies on. A host installs a real adapter through
[`SceneCoordinator::set_hyphenator`](super::SceneCoordinator::set_hyphenator);
this is the [`NullHyphenator`]-equivalent default made explicit so the
byte-identical path takes no extra shaping.
