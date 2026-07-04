//! The default, authoritative contract set for the built-in view primitives
//! (spec §2.6, §2.13).
//!
//! Spec §2.13 requires that unknown attributes and events on a *known*
//! component are compile errors. For that check to bite on a real compile, the
//! [`ContractValidator`](super::ContractValidator) must consult a **populated**
//! registry rather than an empty one; this module supplies the contracts for
//! the minimal built-in primitives so the default compile path enforces their
//! attribute/event surface out of the box.
//!
//! # Why the surface is mirrored here rather than imported
//!
//! The primitives live in `obscur-primitives` as plain view-builder config
//! structs and deliberately **do not** export a component contract (they carry
//! only stable [`ComponentId`]/prop-symbol constants). The typed
//! `obscur_view::component::ComponentContract` names every prop/event by an
//! opaque, hash-derived `InternedSymbol` with no reverse string resolution, so
//! it cannot drive the `.okui` string-keyed lookup either. The authoritative
//! *string* surface for these primitives is therefore their spec-defined
//! builder API (spec §2.6): each contract below mirrors one primitive's
//! attributes/events exactly.
//!
//! [`ComponentId`]: obscur_identity::ComponentId

use crate::validation::ComponentContract;

/// Returns the contracts for every built-in view primitive (spec §2.6), keyed
/// later by their tag name in the [`ContractRegistry`](crate::compile::ContractRegistry).
///
/// The set mirrors the `obscur-primitives` builder surfaces:
///
/// - `Container` — a grouping box: no attributes of its own, any children.
/// - `Text` — a literal text run: no attribute surface (its content is child
///   text), any children.
/// - `Image` — requires `src`, optionally `alt`; a leaf.
/// - `Scroll` — optional `horizontal`/`vertical` axis flags, any children.
/// - `Canvas` — requires `width` and `height`; a leaf.
/// - `Portal` — requires `target`, any children.
/// - `Layer` — optional `opacity`, any children.
/// - `ForeignSurface` — requires `surface`; a leaf.
///
/// A primitive that declares no attributes (`Container`, `Text`) leaves its
/// prop surface *open* — the string contract model treats "no declared props"
/// as unrestricted — so those tags stay permissive; every primitive that
/// declares props closes its surface and rejects unknown attributes.
#[must_use]
pub fn builtin_contracts() -> Vec<ComponentContract> {
    vec![
        ComponentContract::new("Container"),
        ComponentContract::new("Text"),
        ComponentContract::new("Image")
            .requiring_prop("src")
            .allowing_prop("alt"),
        ComponentContract::new("Scroll")
            .allowing_prop("horizontal")
            .allowing_prop("vertical"),
        ComponentContract::new("Canvas")
            .requiring_prop("width")
            .requiring_prop("height"),
        ComponentContract::new("Portal").requiring_prop("target"),
        ComponentContract::new("Layer").allowing_prop("opacity"),
        ComponentContract::new("ForeignSurface").requiring_prop("surface"),
    ]
}

#[cfg(test)]
mod tests {
    use super::{ComponentContract, builtin_contracts};

    #[test]
    fn covers_every_built_in_primitive() {
        let contracts = builtin_contracts();
        let names: Vec<&str> = contracts.iter().map(ComponentContract::component).collect();
        for expected in [
            "Container",
            "Text",
            "Image",
            "Scroll",
            "Canvas",
            "Portal",
            "Layer",
            "ForeignSurface",
        ] {
            assert!(
                names.contains(&expected),
                "missing built-in contract for {expected}"
            );
        }
    }

    #[test]
    fn image_requires_src_and_allows_alt() {
        let contracts = builtin_contracts();
        let image = contracts
            .iter()
            .find(|contract| contract.component() == "Image")
            .expect("an Image contract");
        assert_eq!(image.required_props(), &["src".into()]);
        assert_eq!(image.allowed_props(), &["alt".into()]);
    }

    #[test]
    fn container_leaves_its_prop_surface_open() {
        let contracts = builtin_contracts();
        let container = contracts
            .iter()
            .find(|contract| contract.component() == "Container")
            .expect("a Container contract");
        assert!(container.required_props().is_empty());
        assert!(container.allowed_props().is_empty());
    }
}
