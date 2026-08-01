§6.7 commit handlers for the focused editable's `on-change` command,
called once per edit session with the committed text. A distinct table
from [`text_input_handlers`](Self::text_input_handlers) so an application
can register both on one field without conflation.
