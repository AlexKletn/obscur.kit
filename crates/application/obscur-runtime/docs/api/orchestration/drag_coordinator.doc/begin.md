Begins a drag from `source`.

Returns `false` — starting nothing — when a drag is already in flight: two
concurrent sessions would each stage their own provisional work and there
would be no answer to which one a drop commits.
