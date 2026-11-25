The last **committed** value — the baseline the HTML `change` event
compares against (WHATWG HTML §4.10.5.5 *Common input element APIs*: the
`change` event fires "when the user commits the value" and only when it
*differs* from the value the control had when it gained focus). Seeded to
the field's text at focus, advanced by an explicit commit (single-line
`Enter`) so a later blur without further edits does not re-fire, and
consulted by the runtime's blur/Enter commit path. Distinct from the live
[`buffer`](Self::buffer) text, which every keystroke mutates.
