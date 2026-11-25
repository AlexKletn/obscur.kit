A grammatical gender, as a CLDR value name.

# Why this is not an enum

Because the set is not fixed. `specs/cldr/grammaticalFeatures.xml` gives
Russian `masculine feminine neuter`, Polish `animate inanimate personal
feminine neuter`, and other locales `common` — seven distinct values across
the file, in locale-specific combinations that do not nest. A closed Rust enum
of the three genders Russian happens to have would be exactly the mistake §7.2
is guarding against, one language further along.

So this is an open newtype over a value name, with the seven values CLDR
currently uses as associated constants, and no rejection of anything else. A
gender selects a message arm by name (see
[`MessagePattern`](crate::message::MessagePattern)); the *catalogue* decides
which names its translation distinguishes, because only it knows.
