When `Some(target)`, `stretch` fills `target` and alignment resolves
against `container_cross` (the single-line / no-wrap semantics). When
`None`, the line's cross size is the max item cross size and `stretch`
fills that (the multi-line semantics).
