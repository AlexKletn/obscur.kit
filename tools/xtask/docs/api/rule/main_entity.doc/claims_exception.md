Whether `text` claims the exception, i.e. carries [`MARKER`] as an ordinary
`//` comment.

A plain `//` line and not merely the string anywhere in the file: this rule's
own source spells the marker out in a `const` and in a rustdoc example, and a
substring search would make the checker permanently exempt from itself.
