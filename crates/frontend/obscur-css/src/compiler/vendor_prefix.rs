#![doc = include_str!("vendor_prefix.doc/_module.md")]

#[doc = include_str!("vendor_prefix.doc/OWN_PREFIX.md")]
pub(crate) const OWN_PREFIX: &str = "-obscur-";

#[doc = include_str!("vendor_prefix.doc/VendorNamespace.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VendorNamespace<'a> {
    #[doc = include_str!("vendor_prefix.doc/Own.md")]
    Own,
    #[doc = include_str!("vendor_prefix.doc/Foreign.md")]
    Foreign(&'a str),
}

#[doc = include_str!("vendor_prefix.doc/vendor_namespace.md")]
pub(crate) fn vendor_namespace(name: &str) -> Option<VendorNamespace<'_>> {
    // A custom property is not a vendor one, and the two are told apart by the
    // second character alone: `--x` is an author's own name, `-x-y` is a
    // vendor's. The caller has usually taken custom properties already; this
    // says no again so the classification is true on its own.
    let rest = name.strip_prefix('-')?;
    if rest.starts_with('-') {
        return None;
    }
    let (vendor, tail) = rest.split_once('-')?;
    // `-webkit-` with nothing after it names no property at all. Left
    // unclassified, it falls through to the ordinary unknown-property path,
    // which is the honest answer: it is not a vendor property, it is nonsense.
    if vendor.is_empty() || tail.is_empty() {
        return None;
    }
    // Our prefix ends at the very dash that ends the vendor segment, so the
    // segment already in hand answers the question — and answers it without
    // counting bytes off the front of `name`. An identifier admits every code
    // point at or above U+0080 (css-syntax-3 §4.2), so a byte offset into a
    // property name is not a character offset, and a fixed one lands inside a
    // character as readily as between two.
    if vendor.eq_ignore_ascii_case(OWN_PREFIX.trim_matches('-')) {
        return Some(VendorNamespace::Own);
    }
    Some(VendorNamespace::Foreign(vendor))
}

#[cfg(test)]
mod tests {
    use super::{VendorNamespace, vendor_namespace};

    #[test]
    fn our_own_prefix_is_recognised_as_ours() {
        assert_eq!(
            vendor_namespace("-obscur-painter"),
            Some(VendorNamespace::Own)
        );
    }

    #[test]
    fn a_property_name_is_ascii_case_insensitive() {
        // CSS property names are ASCII case-insensitive (css-syntax-3 §3.3), and
        // a prefix is part of the name.
        assert_eq!(
            vendor_namespace("-OBSCUR-Painter"),
            Some(VendorNamespace::Own)
        );
    }

    #[test]
    fn another_vendors_prefix_names_that_vendor() {
        for (name, vendor) in [
            ("-webkit-box-reflect", "webkit"),
            ("-moz-osx-font-smoothing", "moz"),
            ("-ms-flex", "ms"),
            ("-o-object-fit", "o"),
        ] {
            assert_eq!(
                vendor_namespace(name),
                Some(VendorNamespace::Foreign(vendor)),
                "{name}"
            );
        }
    }

    #[test]
    fn a_vendor_nobody_has_heard_of_is_still_a_vendor() {
        // The rule is syntactic, not a list of known vendors — which is the
        // whole point: a stylesheet written for a toolkit that does not exist
        // yet must still pass through this one untouched.
        assert_eq!(
            vendor_namespace("-someothertoolkit-glow"),
            Some(VendorNamespace::Foreign("someothertoolkit"))
        );
    }

    #[test]
    fn a_vendor_name_whose_bytes_outnumber_its_characters_is_still_read() {
        // css-syntax-3 §4.2 admits every code point at or above U+0080 in an
        // identifier, so a property name's byte offsets are not its character
        // offsets. In `-a-bcdeé` the `é` straddles the eighth byte — exactly
        // where `-obscur-` ends — and reading that many bytes off the front
        // cuts the character in half.
        assert_eq!(
            vendor_namespace("-a-bcdeé"),
            Some(VendorNamespace::Foreign("a"))
        );
        // A near miss on our own prefix: same length in characters, one byte
        // longer, and not ours.
        assert_eq!(
            vendor_namespace("-obscür-glow"),
            Some(VendorNamespace::Foreign("obscür"))
        );
    }

    #[test]
    fn a_custom_property_is_not_a_vendor_property() {
        // `--brand-color` begins with a dash and contains another one, and
        // reading it as vendor-namespaced would silently drop every author
        // variable in the sheet.
        assert_eq!(vendor_namespace("--brand-color"), None);
        assert_eq!(vendor_namespace("--x"), None);
    }

    #[test]
    fn an_ordinary_property_is_in_no_namespace() {
        for name in ["color", "background-color", "grid-template-areas"] {
            assert_eq!(vendor_namespace(name), None, "{name}");
        }
    }

    #[test]
    fn a_dangling_prefix_names_no_property_and_is_not_excused_as_one() {
        // `-webkit-` is not "a vendor property this renderer ignores"; it is a
        // typo, and it should be reported as one.
        assert_eq!(vendor_namespace("-webkit-"), None);
        assert_eq!(vendor_namespace("-"), None);
        assert_eq!(vendor_namespace("-nodash"), None);
    }
}
