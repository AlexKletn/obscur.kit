A [`CustomValueParser`](obscur_style::computed::CustomValueParser) that
remembers what [`lower_custom_value`] already answered.

[`lower_custom_value`] is a pure function of its two arguments: the same text
read against the same property is the same value, in this process, forever.
Nothing about it depends on the sheets in force, on the node being resolved,
or on the order the questions arrive in — which is what makes remembering the
answer sound rather than a cache with an invalidation rule to get wrong.

It is worth remembering because the question is asked far more often than it
has distinct answers. A custom property is declared once and read from every
rule that themes on it: the keyboard's `.key` alone carries fifteen `var()`
declarations, and each is asked again for every key, on every restyle. Without
this, each of those asks ran a CSS tokenizer and a value parser over the same
seventeen bytes and threw the result away — the single largest cost in style
resolution, and the one the deferral in this module introduced.

The table is keyed by property first and text second so a lookup borrows the
text rather than allocating a key to ask with. It is never emptied: an entry
is the answer to a question that cannot change its answer, and the set of
distinct questions is bounded by the declarations the loaded sheets contain.
