Renders one position — the default-slot outlet — with [`Button::contents`] as
its fallback.

Read it as one sentence: *the author's content, or this label if there is
none*. The runtime projects the host's authored children into the outlet
before the output is reconciled, so `<button>Save</button>` keeps its text,
`<button label="Save"/>` gets the fallback, and a node
[`Button::view`] built keeps the very child it was built with — the same
`contents` fragment, put there by the same function.

The context is unused: nothing here reads a signal or asks for a re-render. A
button re-renders when its host node's props or slot content change, which
the component pass already tracks.

[`Button::contents`]: super::Button
[`Button::view`]: super::Button::view
