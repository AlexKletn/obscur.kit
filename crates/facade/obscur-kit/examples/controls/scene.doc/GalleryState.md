The live interaction state the gallery view is parameterized by. Both drivers
own one behind an `Rc<RefCell<_>>`: the command / text-input handlers mutate
it, and the reactive [`view`] builder reads it back to rebuild every frame.

The checkbox and select are kept as their **real widget structs** so their own
state machines ([`Checkbox::toggle`], [`Select::toggle_open`] /
[`Select::select`]) drive the demo; the radio group and input are plain fields
(the radio group is composed, and the input's text is mirrored from the
runtime's editing session).
