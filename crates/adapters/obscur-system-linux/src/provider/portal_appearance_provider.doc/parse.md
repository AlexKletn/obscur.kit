Pick the three keys out of the GVariant text `gdbus` prints.

Textual rather than a real GVariant parse, and deliberately narrow: it finds
each key by name and reads the one value that follows. A reply it does not
recognise yields nothing, which is the same answer as no portal — and the
alternative, a dependency on a D-Bus stack, is not one this adapter takes.
