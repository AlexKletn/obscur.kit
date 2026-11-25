The retained tree's answers about a `<input>` / `<textarea>` field, in the
vocabulary the editing paths ask in.

Six questions — which ancestor is editable, is it multiline, where is its text
leaf, what text does it currently hold, what value does the app declare for it
— and every one of them is a **pure read of the tree**. That is why this holds
no state and takes the tree per call: the editing session is the source of
truth for the *live* buffer, and this is the source of truth for everything the
session has to reconcile itself against.

# Why they are one entity

They share a subject and they share a hazard. Every one of them walks or reads
a node the *tree* owns while the *session* owns the text being displayed, and
the two disagree by design for a keystroke — the session leads, the tree lags.
Keeping the tree-side answers together is what makes it obvious at every call
site which side of that gap is being asked: nothing here ever reads a session,
and nothing here can therefore accidentally answer with a value the user has
already typed past.

The placeholder rule is the entity's own: a `.placeholder` subtree is a
rendering layer and not the field's value, so [`collect_text`](Self::collect_text)
skips it whole. Written anywhere else, an empty placeholdered field would focus
with its placeholder text already in the buffer.
