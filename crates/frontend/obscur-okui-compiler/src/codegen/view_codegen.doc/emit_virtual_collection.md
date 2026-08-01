Emits a `<VirtualFor>` as a virtual-collection marker (spec §2.11, §4.9).

Where [`emit_loop`] emits a real `for` that materializes one node per item, a
`<VirtualFor>` emits a single
[`ViewNode::virtual_collection`](obscur_view::view::ViewNode::virtual_collection)
carrying its row template. The runtime, which owns the viewport, realizes that
marker into a viewport-bounded window — so the generated code never walks the
collection and its size is independent of the collection's length, which is
the promise a `<For>` cannot make.

The window template is emitted **once**, with no item binding: a windowed row
is not yet a function of per-item data (the retained collection consumer the
`collection_rule` docs describe is what would thread it in), so a
`<VirtualFor>` body reads no `item`. Its `empty`/`separator` slots — a `<For>`
affordance about iteration — are not part of a window template and are
dropped.
