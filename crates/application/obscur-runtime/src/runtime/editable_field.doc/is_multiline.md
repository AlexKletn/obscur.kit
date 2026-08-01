Whether the editable `node` is a **multi-line** field (a `<textarea>`): it
carries the reserved CSS type `textarea` (emitted by a multi-line
[`Input`](obscur_widgets::input::Input)). A multi-line editable inserts a
newline on `Enter` and moves the caret vertically on ArrowUp/ArrowDown; a
single-line `input` does neither (its Enter stays activation/submit).
