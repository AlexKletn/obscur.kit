Reports, with the resource's identity attached, an image load/decode that
failed under an installed loader (spec §1.6 "failures should be localized
to the smallest safe scope"; §5.15 loading/ready/**failed** states).

The hub has already recorded the `failed` state and will not retry, so this
fires exactly once per resource, not once per frame.

The record carries the full §1.6 context: the stable `code`
(`resource.load_failed`, machine-readable and identical across locales and
call sites), the missing `resource`'s id, the `node` that referenced it,
the `window` (this runtime's surface) it happened in, and the `recovery`
that kept the frame alive. Only the human `message` is localized — resolved
from the fault class and the runtime's current locale — so the code stays a
stable key while the wording follows the user's language (spec §7.2).
