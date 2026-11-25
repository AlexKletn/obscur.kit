The gradient functions: `linear-gradient()`, its `repeating-` form,
`radial-gradient()` and `conic-gradient()` (CSS Images 3 §3, §4).

Each has its own prelude grammar — an angle or a `to <side>` direction, a
shape/extent/`at <position>` triple, a `from <angle>` — but they converge on
one colour-stop model, and the whole file exists to serve one longhand,
`background-image`. The interior of the entity is the position arithmetic:
stops with omitted positions are distributed evenly, and the repeating form
has to resolve an authored period into an explicit stop list because the
domain models no repetition. Nothing outside a gradient reads any of it.
