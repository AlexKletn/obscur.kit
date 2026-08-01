What a `UiRuntime` is before, during and after one frame — the five claims every
other test file in this directory quietly assumes.

A fresh runtime knows its identity and its viewport and is otherwise **idle**:
nothing pending, no frames counted, no boxes retained. That last clause is the
one worth stating, because a runtime that arrived with boxes would make every
"the frame produced this box" assertion elsewhere unfalsifiable.

After a frame the boxes *are* retained, and at the size and origin the sheet
asked for. Two runtimes handed the same view and the same sheet produce
byte-identical display lists — the determinism the whole suite's paint
comparisons rest on. `set_viewport` is the one input a host changes between
frames, and it reads back.

The last is small and deliberate: `Debug` names the type and the two scheduling
fields, so a panic message or a log line from a host is worth reading.
