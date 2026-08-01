A single authored declaration as written inside a style rule's block, e.g.
`color: red` or `width: 10px !important`.

This is the declaration *before* it acquires any cascade context: it carries
only the target [`PropertyId`], the [`PropertyValue`] to assign, and whether
it was written `!important`. Its [`CascadeOrigin`](crate::cascade::CascadeOrigin),
[`SelectorSpecificity`](crate::selector::SelectorSpecificity) and
`source_order` are not known here — they are supplied at match time by the
[`StyleMatcher`](crate::matching::StyleMatcher), which pairs the origin of
the containing stylesheet, the specificity of the matched selector, and the
declaration's document position to build a full
[`Declaration`](crate::cascade::Declaration).
