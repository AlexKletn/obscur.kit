Puts the computed styles back exactly as they were before
[`apply_presentation`](Self::apply_presentation).

Associated rather than a method: the undo record already names everything it
needs, and the driver may have moved on by the time paint is finished with it.
