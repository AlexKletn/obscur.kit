Everything the Wayland queue dispatches into, and the events it produces.

This is deliberately a plain data sink: the handlers translate Wayland into
the kit's own vocabulary and push onto [`events`](Self::events), and nothing
here draws, lays out or blocks. The host drains the queue after each
dispatch, so a slow frame cannot stall the connection.
