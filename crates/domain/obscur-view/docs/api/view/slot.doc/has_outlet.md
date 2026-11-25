Whether `nodes`, or anything beneath them, contains an outlet (spec §2.12).

[`project_slots`]'s early-out, and the runtime's stray-outlet check: an
outlet in a view that is nobody's render output will never be projected, and
the runtime reports one rather than letting it pass silently.
