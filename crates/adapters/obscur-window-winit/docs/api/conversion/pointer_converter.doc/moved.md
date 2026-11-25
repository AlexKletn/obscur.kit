Builds a [`PointerEventKind::Move`] event at `position` for pointer `id`, with
the `held` buttons it travelled under (spec §4.7).

A move transitions no button, so it names none — but it does carry the set,
which is what makes `is_pressed` answer on the event a custom drag is written
against. winit reports a move and a button in two different events, so only a
caller tracking the set across both can supply this.
