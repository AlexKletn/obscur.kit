Whether propagation should continue **upward** to the parent context.

Per §4.3, propagation continues only when the output geometry changed
(`geometry_changed`) **and** this context does not contain layout
(`!contained`). A containment boundary halts propagation even on a
geometry change, and stable geometry halts it regardless of containment.
