Mounts the built-in text-entry control under [`Input::COMPONENT_ID`].

One registration, two tags: `<input>` and `<textarea>` are the same id here
(HTML's two text-entry elements are one widget, `Input::multiline`), and the
distinction between them rides on the local name rather than on a second
component. So this factory answers for both, and the contract it hands back
is one contract.

[`Input::COMPONENT_ID`]: super::Input::COMPONENT_ID
