Returns the compiled built-in **user-agent (UA) default stylesheet** — the
toolkit's neutral, overridable base component look, at the lowest cascade
origin ([`CascadeOrigin::SystemDefault`](crate::style::cascade::CascadeOrigin::SystemDefault)).

See [`crate::default_stylesheet`] for the override contract, and
`obscur_theme_system::stylesheet::ua_css` — which it re-exports — for the raw
CSS.
