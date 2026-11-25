Rejects a Rust expression outside the pure-read subset §2.10 allows, returning
why.

# One rule, both front ends

This is *the* §2.10 check. The `okui!` macro calls it with the
[`syn::Expr`](syn::Expr) it already parsed; a compiled `.okui` file reaches it
through [`ExpressionValidator::validate`], which parses the attribute's text
first. Two copies of a security rule is one copy too many — they drift, and the
looser one is the one that decides.

# A positive allow-list, not a deny-list

The match names what an expression **may** be and refuses everything else, so a
shape nobody thought about is refused rather than admitted. That direction is
the point: §2.10's forbidden side names *effects* — "I/O", "direct service
access" — and no syntactic list can enumerate those. A list of what is allowed
is only ever as good as the spec; a list of what is forbidden is only ever as
good as the imagination of whoever wrote it.

It has teeth only since bindings became real Rust. A binding used to be emitted
as a string, so `{do_io()}` produced the harmless text `"do_io()"` — the rule
was policing a language that never evaluated.

# What is allowed, and why each

* **paths** and **field access** — §2.10's props, read-only state, loop locals
  and field access. The whole point of a binding.
* **literals** — a constant is a read of nothing.
* **comparisons and logical operators**, and `!` / `-` — named by §2.10.
* **arithmetic** — named by neither side, and allowed because every §2.10
  *forbidden* item is an effect and arithmetic has none. Refusing `{a + b}`
  while allowing `{a > b}` would be a rule about spelling rather than about
  what an expression does.
* **parentheses**, **indexing**, **casts**, **borrows** — grouping and reads by
  another name.
* **method calls** — the gap, below.

Everything else is refused, which by construction covers §2.10's forbidden
list: assignment and compound assignment, `await` / `async`, closures,
`unsafe`, macros, loops, `?`, and **free function calls** — the "unconstrained
arbitrary function calls" it names, refused until something declares which
functions are its "explicitly exported pure" ones.

# The method-call gap, and the two edges of it §2.10 *does* let us cut

A method call is where Rust spells three things §2.10 *allows* — Option/null
checks (`.is_some()`), formatting and translations — and several it *forbids*.
Nothing syntactic tells `.format()` from `.write_to_disk()` in general, so the
receiver is admitted by default. But two of the forbidden bullets have a shape
this can recognise, and refusing them wholesale would not forbid anything the
spec allows:

* **arbitrary mutation** — a method whose name is a standard in-place mutator
  (`push`, `sort`, `insert`, `clear`, …; see [`is_mutating_method`]). An
  author reads state in an expression; they do not mutate it there.
* **direct service access** — a method invoked on a value reached *through* a
  field (`services.http.get(...)`), as opposed to called directly on a read
  binding (`user.is_some()`). This is a structural approximation, not a proof:
  it is the shape "navigate into an object, then call into it" that §2.10's
  "direct service access" names, and it leaves the direct-on-binding calls the
  spec allows untouched.

  Because it is an approximation, it also refuses calls that are no service at
  all — `row.keys.len()` is the same shape as `services.http.get(url)`, and
  nothing syntactic separates them. The message says exactly that rather than
  asserting the call *is* service access; telling an author their `.len()` is a
  service sends them hunting for one they never wrote.

The receiver and the arguments are still walked, so a free call cannot hide
inside one.

# Free calls and the exported-pure exception

A **free function call** (`Expr::Call`) is §2.10's "unconstrained arbitrary
function calls" — refused, *unless* the callee is a bare name that a `Function`
symbol declared in scope, which is §2.10's Allowed "explicitly exported pure
functions." The set of such names is [`functions`](check_expression_in); the
context-free [`check_expression`] passes an empty one, so the macro front end
(which has no way to declare a pure function) refuses every free call, as it
did before this exception existed.

# Errors

Returns the reason `expr` is outside §2.10's subset — a pointed message for an
assignment, mutation, service access or refused call, else
[`FORBIDDEN_MESSAGE`]. The `Err` is a sentence for a caller to render, since
the two callers report it differently: the macro attaches a span, the file
path a byte offset.
