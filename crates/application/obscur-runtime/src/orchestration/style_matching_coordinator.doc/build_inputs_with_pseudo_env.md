Matches `sheets` against every element node in `tree`, honouring both the
interaction state in `pseudo` and the media/container `environment`.

This is the fully-parameterised worker the runtime frame path uses: each
element is matched with [`StyleMatcher::match_element_in`], so a rule that
was authored inside an `@media`/`@container` group contributes only when
its condition holds against `environment` (§4.13), while `:hover`/`:focus`
are honoured against `pseudo`. Passing [`MediaEnvironment::unbounded`] is
exactly equivalent to [`build_inputs_with_pseudo`](Self::build_inputs_with_pseudo).
