Installs a `hyphenator` on the owned scene coordinator, replacing the
default no-op [`NullHyphenator`](obscur_text::layout::NullHyphenator).

Hyphenation is a hexagonal port: the runtime only knows the
`Box<dyn Hyphenator>` — it never depends on a concrete pattern set. A host
installs a real adapter (e.g. Knuth-Liang English patterns) here; from the
next laid-out frame on, an over-wide word breaks at a valid point and (when
a hyphen glyph is shaped) renders a trailing hyphen. The **same** port
sizes text in the layout pass and paints it in the compose pass, so the
measured box height and the painted line count agree. Leaving it unset
preserves headless determinism, so all default-path tests are unchanged.
