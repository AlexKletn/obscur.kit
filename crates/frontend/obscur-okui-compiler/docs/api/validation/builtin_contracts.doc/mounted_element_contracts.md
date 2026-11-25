The §2.13 child policy of the **base-element** spellings whose registered
factory owns their children.

The lowercase control tags are otherwise unvalidated — the contract registry
is keyed by the authored tag string, and `<Checkbox/>` requires a `label`
while `<checkbox/>` does not. That permissiveness is deliberate, and this is
the one part of it that could not survive `BuiltinComponents`: once a factory
mounts on a tag, the component's `render` output **replaces** the host's
children, so an authored child under one of these tags is discarded by the
frame that renders it. Discarded silently, and by a registration made
somewhere else entirely.

So the rule these five entries state is not "these tags are validated" but:

> a base element whose registered factory owns its children forbids authored
> children.

`<textarea>Hello</textarea>` is the case that forced it — HTML makes a
textarea's child text its *default value*, and this toolkit spells that
`value="…"`. Rejecting it names the migration; rendering an empty field
would not.

`<table>` and `<button>` are deliberately absent, for two different reasons
that both end in "their children still render". Nothing mounts on `<table>`
at all. A factory *is* registered for `<button>`, but its component owns no
children: it renders the §2.12 default-slot outlet, so the author's content
is projected straight back into it — which is precisely why the sentence
above says "whose factory **owns** its children" rather than "on which a
factory is registered". Adding a policy for either would forbid markup that
works.

Only the child policy is stated. Declaring a single prop would **close** the
prop surface (see `check_props`), which would reject attributes these tags
accept today — a much larger change than the one this exists to make.
