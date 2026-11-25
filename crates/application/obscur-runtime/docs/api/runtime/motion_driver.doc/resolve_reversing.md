Resolves the effective duration, delay and reversing bookkeeping for a CSS
transition about to start on `key`, applying css-transitions-1 §3.1
"Faster reversing of interrupted transitions".

`previous` is the before-change value (the change-detection baseline) and
`after` is the after-change value. When a transition is already running for
`key` and its reversing-adjusted start value equals `after`, the change
*reverses* it: the new reversing shortening factor is the clamped absolute
value of `old_output · old_factor + (1 − old_factor)`, where `old_output`
is the running transition's timing-function output at this instant, and the
matching duration — plus any *negative* delay — is multiplied by that
factor. Otherwise the matching duration and delay are used unchanged and
the transition starts with a reversing shortening factor of `1`.
