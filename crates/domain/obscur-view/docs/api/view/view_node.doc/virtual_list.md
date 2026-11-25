Creates a virtual-list marker: `count` items of `row_extent` each, realized
through the row builder registered under `list_id` (spec §4.9).

The runtime holds the registered builder and this marker names it. Use
[`Self::virtual_list_with_rows`] for a marker that owns its builder.
