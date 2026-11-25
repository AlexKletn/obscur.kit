Returns whether this unit snaps the caret to a fixed edge rather than
advancing it a countable amount.

True for [`LineBoundary`](Self::LineBoundary) (Home/End) and
[`Document`](Self::Document) (Ctrl+Home/End): both jump to an edge, so a
repeated move in the same direction is idempotent, whereas the other
units keep advancing.
