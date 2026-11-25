Phase 3 (§3.4): dispatches every queued input event, in order, through the
same entry points an immediate caller uses.

Returns whether anything was dispatched. Runs before the frame commits or
chooses its plan, so what dispatch changes — a pseudo-state, focus, the
editing session — is what this frame goes on to render, and the frames it
asks for are folded into this one (see [`request_frame`](Self::request_frame)).
