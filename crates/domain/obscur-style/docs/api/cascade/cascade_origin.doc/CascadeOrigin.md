The origin of a declaration in the cascade, ordered by ascending precedence
for *normal* (non-`!important`) declarations (§4.14).

The six origins, from lowest to highest precedence for normal declarations:

1. [`CascadeOrigin::SystemDefault`] — the built-in system default styles;
2. [`CascadeOrigin::Theme`] — the active theme stylesheet;
3. [`CascadeOrigin::GlobalApp`] — global, application-wide CSS;
4. [`CascadeOrigin::ComponentModule`] — component / module-scoped CSS;
5. [`CascadeOrigin::Inline`] — inline CSS set directly on a node;
6. [`CascadeOrigin::RuntimeOverride`] — an explicit runtime override layer.

The variants are declared in this ascending order and the type derives
[`Ord`], so the derived ordering *is* the normal-declaration precedence:
`SystemDefault < Theme < GlobalApp < ComponentModule < Inline <
RuntimeOverride`.

Note that this ordering only directly governs *normal* declarations; how
`!important` and the runtime-override layer interleave with authored CSS is
handled by [`crate::cascade::CascadePriority`], which builds its band from
both the origin and the declaration's `!important` flag.
