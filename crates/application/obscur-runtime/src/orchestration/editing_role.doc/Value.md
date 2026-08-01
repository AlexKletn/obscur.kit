`node` is the field's ordinary editable text (the field itself, or a `Text`
descendant that is *not* under a `.placeholder` overlay): the session
**always** overrides it, even when the session text is empty (an emptied
field renders empty, byte-identical to the historical value path).
