The reserved child tags of a `<For>` that are slots rather than the item
template (spec §2.11).

`<Loading>` is **not** here. `empty` and `separator` are decidable from the
collection itself — it has a length, and iteration has a between — but nothing
about a Rust collection says it is still loading: that needs a signal from
outside, and the spec does not say what it is. Inventing one (`loading={…}`,
say) would be putting words in the spec's mouth, so the slot stays unparsed
and `<Loading>` is treated as an ordinary child until the signal is defined.
