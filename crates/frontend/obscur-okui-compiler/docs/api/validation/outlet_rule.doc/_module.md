The §2.12 outlet rule: what may be written on an authored `<slot …/>`, stated
over plain data so **both** front ends decide it the same way.

# What an outlet is, and what that costs it

`<slot name="header"/>` is a *position*, not an element:
`project_slots` deletes it and splices in every authored child assigned
`slot="header"`. Nothing an outlet carries survives that substitution — the
node the author wrote is gone before style, layout or paint can see it.

That is what makes every rule below a rule rather than a preference. A
`class="banner"` on an outlet is not "ignored"; it is *deleted*, silently, and
the author is left reading a stylesheet that never matched anything. §2.13 says
unknown attributes are compile errors; on an outlet, *every* attribute but
`name` is unknown, because there is no node left to carry it.

# The rules

- **`name` is required.** An outlet with no name names no slot, so
  `project_slots` fills it with nothing and deletes it. It is a slot
  declaration that can never be satisfied — exactly the "inert box" this
  refuses at the tag instead of at the frame.
- **`name` must be a literal.** `name={expr}` is refused for the reason
  `slot={expr}` already is
  ([`DynamicSlot`](crate::validation::ContractViolation::DynamicSlot)): a slot
  is matched against a declaration at *compile* time, and a name that does not
  exist until runtime cannot be. The two halves of one join must be decidable
  in the same pass, or neither is.
- **Nothing else may be written on it.** Not `class`, not `id`, not `on:click`,
  not `key` — see above.
- **It takes no children.** HTML's `<slot>` renders its children as fallback
  content when nothing is assigned. §2.12 describes no fallback content, and
  [`project_slots`] implements none: an unfilled outlet leaves *nothing*
  behind. Accepting children would mean silently dropping them. Inventing
  fallback semantics because HTML has them is the move `5e8e7fe` refused for
  `<Loading>`; refusing the spelling is what this does instead.

# One rule, two front ends

The rule takes plain data and returns messages, with no syntax in it, for the
same reason [`check_collection`](crate::validation::check_collection) and
[`check_contract`](crate::validation::check_contract) do: there are two
authoring surfaces and only one has a `.okui` syntax tree. Each front end
attaches its own location — a byte offset for the file path, a `Span` for the
`okui!` macro.

# What this rule cannot decide

Whether the outlet is in a **component's render at all**. A component's `render`
returns a `View` and `okui!` produces a `View`, so the identical
`okui! { <slot name="header"/> }` is correct inside `render` and stray one
function away, and a proc-macro cannot see which of the two it was expanded in.
Nor can it check the name against the component's declared slots: the
declaration lives on `MyProps::contract_surface()`, a function that runs when
the built crate runs, while this runs during that crate's compilation — the same
wall that keeps an unregistered `<Crad/>` silent. The stray case is decided at
runtime instead, where the answer is knowable: the reconciler refuses to
materialize an outlet and `UiRuntime` reports it.

[`project_slots`]: obscur_view::view::project_slots
