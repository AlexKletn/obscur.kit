Realizes every virtual-collection marker in `view` into a viewport-bounded
window of rows (spec §2.11, §4.9).

Each marker is replaced, in place and in order, by `window_len(extent)`
clones of its template children, where `extent` is the marker's estimated row
extent (defaulting to [`DEFAULT_VIRTUAL_ROW_EXTENT`]). The caller supplies
`window_len` — the runtime computes it from the live viewport through
[`obscur_layout::virtualization::VirtualLayout`], which is what makes the
realized count bounded by the viewport and independent of the collection's
length.

`view` is returned untouched, without walking or rebuilding it, when it holds
no marker — which is every document that authors no `<VirtualFor>`.
