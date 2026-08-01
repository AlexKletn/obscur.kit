The pushed half: events this host produces for subscribed DevTools
clients. The frame thread pushes; the serving thread fans out through the
endpoint, so it — not this queue — still decides who may hear each one.
