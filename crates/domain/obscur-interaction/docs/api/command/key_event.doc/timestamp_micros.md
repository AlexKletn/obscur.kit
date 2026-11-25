Microseconds on whatever monotonic origin the host chose, matching
`PointerEvent::timestamp_micros` exactly.

Zero means unstamped. The runtime never reads a clock; time is data.
