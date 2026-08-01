Registers (or replaces) the host-provided [`FontData`] for the run font
handle `id` (`0` = primary; higher ids are fallback faces).

Fonts are a host resource; the adapter never fabricates one. The `id` must
match the face id the runtime's font resolver assigned (so scene run
handles line up with these fonts). Until a covering font is registered,
the run's [`PaintCommand::DrawText`] is skipped.
