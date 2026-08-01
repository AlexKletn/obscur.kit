Returns which children the contract admits (spec §2.13).

This replaced an `allowed_children() -> &[Box<str>]` accessor whose empty
slice meant "any child": there was no answer it could give for a leaf, and
every caller that read emptiness as permission was reading the bug. Ask the
[`ChildPolicy`] instead — [`admits`](ChildPolicy::admits) for an element
tag, [`admits_content`](ChildPolicy::admits_content) for text.
