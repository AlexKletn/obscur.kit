§5.12 IME composition: the host's composition events applied to the focused
editable's session, and the live preedit a renderer reads back out.

The two belong together because the preedit is the thing that makes composition
different from typing. It is shown inline and is deliberately **never** written
into the buffer, so the committed text a handler sees is unchanged for the whole
time the user is composing — and the accessor here is the only way anything
outside the session can see the in-flight text at all. A commit then replaces
the anchor span as one edit, so one composition is one undo unit.

The buffer surgery is [`TextEditSession`](crate::runtime::TextEditSession)'s;
what is here is the mapping from the four host events onto it, and the follow-up
the runtime owes afterwards — a paint frame, or the single edit that a commit
becomes. The follow-up is resolved while the session borrow is live and acted on
after it ends, which is why it is a value rather than a call.
