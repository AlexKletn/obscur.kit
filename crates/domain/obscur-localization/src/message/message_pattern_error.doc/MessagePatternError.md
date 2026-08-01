Why [`MessagePattern::parse`](crate::message::MessagePattern::parse) refused a
pattern.

Parsing fails loudly rather than degrading, because a catalogue is *data* —
often data a translator edited — and a pattern that half-parses paints its own
braces on the screen. A caller checks this once at load, not once per frame.
