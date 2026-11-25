Reserved symbol for a virtual collection's **estimated row extent** — the
per-item size, in logical pixels along the scroll axis, the window is sized
against until real measurement refines it (spec §4.9 *estimated geometry*).

Like [`SLOT`](super::SLOT) it lives in the structural `0x0000_03xx`
namespace, distinct from the CSS-relevant reserved attributes
(`0x0000_000x`), the reserved events (`0x0000_010x`) and the stable selectors
(`0x0000_02xx`): it is a realization input, not a CSS selector input, so no
selector can match on it.
