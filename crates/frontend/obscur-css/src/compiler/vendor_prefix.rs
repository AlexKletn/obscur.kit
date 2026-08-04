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
    if name.len() >= OWN_PREFIX.len() && name[..OWN_PREFIX.len()].eq_ignore_ascii_case(OWN_PREFIX) {
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
