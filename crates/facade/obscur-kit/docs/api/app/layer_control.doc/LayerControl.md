A handle an application keeps to the desktop surface the loop owns.

Two directions. **Outward**: [`reconfigure`](Self::reconfigure) and
[`close`](Self::close) leave a request the loop picks up at the end of its next
turn — a handler cannot touch the surface directly because the loop has it
borrowed. **Inward**: [`size`](Self::size), [`scale`](Self::scale) and
[`state`](Self::state) report what the platform last settled on, which is the
only authoritative answer, since a bar asks for a height and gets told a width.

Cheap to clone, and every clone is the same surface.
