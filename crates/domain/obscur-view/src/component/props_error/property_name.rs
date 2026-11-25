use core::fmt;

use obscur_identity::InternedSymbol;

/// A property named as helpfully as the caller made possible: in words and by
/// number when a label travelled with it, by number alone when none did.
pub(super) struct PropertyName<'a>(pub(super) InternedSymbol, pub(super) &'a str);

impl fmt::Display for PropertyName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The number is kept even when the name is known: it is what every other
        // report about this property prints, and a reader correlating two of
        // them needs the two to agree on something.
        if self.1.is_empty() {
            write!(f, "#{}", self.0.raw())
        } else {
            write!(f, "`{}` (#{})", self.1, self.0.raw())
        }
    }
}
