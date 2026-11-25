The files over [`MAX_NON_TEST_LINES`] today, each with the reason it is still
there. **It is empty**: `ui_runtime.rs`, the god object this register was
created for, came under the cap when its input surface moved into child modules,
and the entry was deleted the same pass — which is what the rule below requires.

**This list is a debt register, not a permission slip.** A new entry needs a
reason a reviewer accepts; an entry that shrinks under the cap must be
deleted, so a file cannot quietly grow back. Paths are workspace-relative and
use `/` on every platform.
