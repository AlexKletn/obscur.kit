Interned name of the `modal` state property.

It is [`FOCUS_TRAP`](obscur_interaction::focus::FOCUS_TRAP), not a name of this
widget's own, and that is the whole of what makes a modal dialog modal: the
runtime reconciles a trapping focus scope over every element carrying that
property, so the trap opens when this element enters the committed tree and
closes when it leaves. A private symbol here is what the property used to be, and
it was read by nothing — an open dialog let Tab walk straight out into the page
behind it.
