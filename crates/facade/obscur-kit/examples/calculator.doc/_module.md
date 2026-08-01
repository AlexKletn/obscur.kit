A touch-first calculator — the kit's gesture stack in something you can
actually use with a thumb.

It exists because a calculator is the smallest honest test of touch input:
every interaction is a *tap on a small target*, which is where a pointer stack
either works or is exhausting. Everything the kit grew for touch shows up here
in the shape an application would really use it:

- **Taps with feedback.** Keys are 80×68 (comfortably past a fingertip's ~9mm)
  and light `:active` while held. The runtime's press state is per-pointer, so
  resting one thumb on a key and pressing another with the other hand lights
  both and fires both.
- **A held key means something else.** `⌫` removes one digit on a tap and
  clears everything when held — a touchscreen has one button and no
  modifiers, so holding is the only "other" gesture a key has. It fires while
  the finger is still down, and the lift is consumed rather than also
  deleting a digit.
- **A flickable tape.** The history is a kit-owned virtual list: drag it with
  a finger, flick it and it coasts to a stop, catch it and it stops dead
  without also recalling whatever line was sliding past. Tapping a line
  recalls its result; holding a line deletes it.
- **Insets.** The root pads itself by `env(safe-area-inset-*)`, so on a device
  with a notch or a gesture bar the keys are not under them. On a desktop
  every inset is zero and the padding costs nothing.

Run it on a real machine (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example calculator --features full`

It cannot run in the headless dev/CI environment (no display, no GPU), so
there it is only compiled. The arithmetic is a plain state machine and *is*
testable there — `cargo test -p obscur-kit --features full --example
calculator` runs the cases at the bottom of this file. An example's tests do
not run in the workspace sweep, so that command is the only thing that runs
them.
