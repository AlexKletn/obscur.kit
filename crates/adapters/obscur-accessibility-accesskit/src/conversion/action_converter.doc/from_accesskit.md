Maps an inbound AccessKit [`Action`] back to the domain
[`SemanticAction`] it should dispatch, or [`None`] when the domain has no
equivalent.

[`Action::Click`] maps to [`SemanticAction::Activate`]. AccessKit actions
outside the shared set — the directional scroll actions, `Blur`,
`CustomAction`, text-selection and tooltip actions, and so on — return
[`None`].
