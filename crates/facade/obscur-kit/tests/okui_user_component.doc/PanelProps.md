A `Panel` that renders a **classed element**, not a bare text node, and
declares §4.15's explicitly-open style scope so the application's sheets
reach it.

`Card` above cannot carry these assertions: it renders one text node, and a
text node has no CSS identity at all. That is not an accident of the fixture
— it is the reason the finding survived this file.
