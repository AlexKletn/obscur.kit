What a text field *tells the app*, and when — the outside of text entry, with
the editing session deliberately left inside.

Twelve tests over one distinction that WHATWG HTML §4.10.5.5 draws and that is
easy to lose: `input` fires **per keystroke** with the running value, `change`
fires **once on commit** with the committed one. They are separate registries
keyed by the same command symbol, because the `Input` widget binds both events
to one symbol — so an app can take one, the other, or both, and the last test
holds that separation directly.

The five `input` tests are the dispatch path: typing into a focused editable
reaches the app's own string and the reactive rebuild shows it; a **placeholder**
is an overlay and must never be seeded into the buffer; a Ctrl chord is a
shortcut and not text; a char over a focused *non*-editable dispatches no text
at all; and a bound field with no registered handler still consumes the key
rather than letting it fall through to the shortcut path.

The five `change` tests are the commit path, and four of them are about the
event *not* firing: not on the focus click, not on a second click into the same
field, not on a blur that changed nothing, and not on a second `Enter` after the
first one already committed. A `change` that fires on focus was a real defect
here once, which is why the click case is written down as its own test.

The remaining two ask the question a host asks before raising an on-screen
keyboard — does the thing holding focus actually take typed text? A button takes
focus exactly like a field does, so "something is focused" is not the answer;
"the focused thing binds `EVENT_INPUT`" is, and a wrong `true` is a keyboard
covering half a tablet in an app with no field.
