Whether what currently holds focus can accept typed text — i.e. whether the
host should be offering its text-entry facility right now (spec §5.12).

# Why a host asks

On a desktop this decides whether an IME is armed, which is invisible when
the answer is wrong. On a tablet it decides whether **the on-screen
keyboard covers half the screen**, which is not invisible at all: a host
that offers text entry unconditionally raises a keyboard over a calculator
that has no field to type into. That is what this exists to let a host
avoid, and it is a *question about focus*, so the runtime — which owns
focus — is what answers it rather than each host guessing.

The test is the same one the editing session uses: a focused node whose
ancestor chain binds [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT) is a text
field, and nothing else is.
So it follows the real editables automatically — a widget that becomes
editable, or a field that is replaced by a label, needs no second list to
be kept in step with this one.
