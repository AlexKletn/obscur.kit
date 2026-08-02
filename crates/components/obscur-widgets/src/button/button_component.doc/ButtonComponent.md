The shipped `button` as a mounted component — the one whose content belongs
to the author.

Every other shipped control renders content the author cannot write: a
checkbox's indicator, a select's popup rows, a field's editable run. A button
renders a label, and a label is markup — `<button>Save</button>` puts it in
the node's *children*, where a component's own render would ordinarily
replace it. Mounting this control naïvely would empty every authored button
in every existing program, which is why nothing was mounted on this id until
there was an answer.

The answer is §2.12. This component renders exactly one node, and that node
is not content at all: it is the **default-slot outlet**, the position that
says "the author's content goes here", carrying [`Button::contents`] as its
fallback for the `<button label="Save"/>` spelling that supplies none. So the
control is mounted — its contract, its id and its factory are real — and what
it renders is still what was written.

It is a unit type, and that is a claim rather than an omission: a button holds
no state its host node does not already carry. Its label, its pressed and
disabled flags and its command are all on the node the runtime is rendering.

[`Button::contents`]: super::Button
