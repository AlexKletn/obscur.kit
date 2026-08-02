Whether the element's subtree renders no text at all — every descendant text
node is absent or empty.

This is the "is the field empty?" answer for a node whose value was never
declared as a reserved `VALUE` attribute, which is every node authored as
markup: `<input value="abc"/>` is HTML's *default* value (WHATWG HTML
§attr-input-value), owned by the user after the first keystroke, so it must not
lower to the controlled-value attribute that would snap a drifting buffer back.
It reaches the tree as the editable run the mounted control renders, and that
run is the only place the value can be read from.

The walk mirrors the one the editing session seeds from (`EditableField`), which
is why it descends rather than looking only at direct children: a field's
editable text is a text node somewhere under the host, not necessarily its first
child.
