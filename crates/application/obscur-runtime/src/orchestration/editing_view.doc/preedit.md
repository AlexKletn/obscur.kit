The active IME preedit spliced into the field for DISPLAY when a
composition is live (spec §5.12), or `None` when the field is not
composing. When present, the compose phase draws `text` with the preedit
spliced at [`PreeditView::at`], underlines the preedit span, and places the
caret at the preedit's caret — none of which touches the buffer.
