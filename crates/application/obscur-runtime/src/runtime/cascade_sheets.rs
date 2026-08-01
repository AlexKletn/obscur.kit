#![doc = include_str!("cascade_sheets.doc/_module.md")]

use obscur_style::matching::Stylesheet;

#[doc = include_str!("cascade_sheets.doc/AuthorSheets.md")]
pub(crate) struct AuthorSheets(Vec<Stylesheet>);

#[doc = include_str!("cascade_sheets.doc/CascadeSheets.md")]
#[derive(Debug)]
pub(crate) struct CascadeSheets {
    #[doc = include_str!("cascade_sheets.doc/reactive.md")]
    reactive: Vec<Stylesheet>,
    #[doc = include_str!("cascade_sheets.doc/retained_author.md")]
    retained_author: Vec<Stylesheet>,
    #[doc = include_str!("cascade_sheets.doc/default_sheet.md")]
    default_sheet: Option<Stylesheet>,
}

impl CascadeSheets {
    #[doc = include_str!("cascade_sheets.doc/new.md")]
    pub(crate) const fn new() -> Self {
        Self {
            reactive: Vec::new(),
            retained_author: Vec::new(),
            default_sheet: None,
        }
    }

    #[doc = include_str!("cascade_sheets.doc/set_reactive.md")]
    pub(crate) fn set_reactive(&mut self, sheets: Vec<Stylesheet>) {
        self.reactive = sheets;
    }

    #[doc = include_str!("cascade_sheets.doc/take_reactive.md")]
    pub(crate) fn take_reactive(&mut self) -> Vec<Stylesheet> {
        std::mem::take(&mut self.reactive)
    }

    #[doc = include_str!("cascade_sheets.doc/set_default.md")]
    pub(crate) fn set_default(&mut self, sheet: Stylesheet) {
        self.default_sheet = Some(sheet);
    }

    #[doc = include_str!("cascade_sheets.doc/has_default.md")]
    pub(crate) const fn has_default(&self) -> bool {
        self.default_sheet.is_some()
    }

    #[doc = include_str!("cascade_sheets.doc/author_for_frame.md")]
    pub(crate) fn author_for_frame(&mut self, supplied: &[Stylesheet]) -> AuthorSheets {
        if supplied.is_empty() {
            return AuthorSheets(self.retained_author.clone());
        }
        self.retained_author = supplied.to_vec();
        AuthorSheets(supplied.to_vec())
    }

    #[doc = include_str!("cascade_sheets.doc/cascade.md")]
    pub(crate) fn cascade(&self, author: AuthorSheets) -> Vec<Stylesheet> {
        let AuthorSheets(mut sheets) = author;
        if let Some(default) = &self.default_sheet {
            sheets.insert(0, default.clone());
        }
        sheets
    }
}

#[cfg(test)]
mod tests {
    use super::CascadeSheets;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule, Stylesheet};
    use obscur_style::property::{Length, PropertyId, PropertyValue};
    use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};

    /// A one-rule sheet at `origin` selecting `ty`, so two sheets built from
    /// different arguments are never accidentally equal.
    fn sheet(origin: CascadeOrigin, ty: &str) -> Stylesheet {
        Stylesheet::new(
            origin,
            vec![StyleRule::new(
                SelectorList::new(vec![ComplexSelector::new(
                    CompoundSelector::new(vec![SimpleSelector::Type(ty.into())]),
                    Vec::new(),
                )]),
                vec![BlockDeclaration::normal(
                    PropertyId::Width,
                    PropertyValue::Length(Length::px(10.0)),
                )],
            )],
        )
    }

    fn author(ty: &str) -> Stylesheet {
        sheet(CascadeOrigin::GlobalApp, ty)
    }

    fn ua() -> Stylesheet {
        sheet(CascadeOrigin::SystemDefault, "button")
    }

    /// The whole resolution in one line, as a frame performs it.
    fn frame(sheets: &mut CascadeSheets, supplied: &[Stylesheet]) -> Vec<Stylesheet> {
        let resolved = sheets.author_for_frame(supplied);
        sheets.cascade(resolved)
    }

    #[test]
    fn a_runtime_with_no_sheets_cascades_nothing() {
        let mut sheets = CascadeSheets::new();
        assert!(!sheets.has_default());
        assert!(
            frame(&mut sheets, &[]).is_empty(),
            "an initial bare frame retains nothing, so it stays genuinely unstyled"
        );
    }

    #[test]
    fn the_ua_sheet_is_matched_below_every_author_sheet() {
        let mut sheets = CascadeSheets::new();
        sheets.set_default(ua());
        let cascade = frame(&mut sheets, &[author("a"), author("b")]);
        assert_eq!(
            cascade,
            vec![ua(), author("a"), author("b")],
            "UA first, author order preserved behind it"
        );
    }

    #[test]
    fn without_a_ua_sheet_the_author_list_is_handed_through_untouched() {
        let mut sheets = CascadeSheets::new();
        let supplied = vec![author("a"), author("b")];
        assert_eq!(frame(&mut sheets, &supplied), supplied);
    }

    #[test]
    fn a_bare_frame_re_presents_the_last_styled_frames_sheets() {
        // The defect retention exists for: a scroll or a hover re-rendered with
        // `&[]` must cascade what the styled frame before it cascaded, not the UA
        // default alone.
        let mut sheets = CascadeSheets::new();
        sheets.set_default(ua());
        let styled = frame(&mut sheets, &[author("a")]);
        assert_eq!(frame(&mut sheets, &[]), styled);
        assert_eq!(frame(&mut sheets, &[]), styled, "and every one after it");
    }

    #[test]
    fn the_ua_sheet_never_accumulates_across_bare_frames() {
        // The half the `AuthorSheets` token guards: what is remembered is the
        // author list, so however many bare frames follow a styled one, the
        // cascade carries exactly one UA sheet.
        let mut sheets = CascadeSheets::new();
        sheets.set_default(ua());
        let _ = frame(&mut sheets, &[author("a")]);
        for _ in 0..3 {
            let cascade = frame(&mut sheets, &[]);
            assert_eq!(
                cascade.iter().filter(|s| **s == ua()).count(),
                1,
                "one UA sheet, not one per bare frame"
            );
        }
    }

    #[test]
    fn a_replaced_ua_sheet_replaces_it_on_a_bare_frame_too() {
        // The failure the remembered-cascade-list shape would cause: a retained
        // copy of the old UA sheet outliving the sheet that replaced it, and
        // winning every tie against it by sitting later in source order.
        let mut sheets = CascadeSheets::new();
        sheets.set_default(ua());
        let _ = frame(&mut sheets, &[author("a")]);
        let replacement = sheet(CascadeOrigin::SystemDefault, "input");
        sheets.set_default(replacement.clone());
        assert_eq!(
            frame(&mut sheets, &[]),
            vec![replacement, author("a")],
            "the bare frame cascades the new UA sheet and no trace of the old one"
        );
    }

    #[test]
    fn a_supplied_set_replaces_the_retained_one_rather_than_adding_to_it() {
        let mut sheets = CascadeSheets::new();
        let _ = frame(&mut sheets, &[author("a"), author("b")]);
        assert_eq!(frame(&mut sheets, &[author("c")]), vec![author("c")]);
        assert_eq!(frame(&mut sheets, &[]), vec![author("c")]);
    }

    #[test]
    fn an_empty_set_does_not_clear_the_retained_one() {
        // Stated as behaviour, not endorsed: supplying `&[]` is how a frame says
        // "I own no styling", so it cannot also be how a host says "drop the
        // styling". A host that means the latter supplies a sheet with no rules.
        let mut sheets = CascadeSheets::new();
        let _ = frame(&mut sheets, &[author("a")]);
        sheets.set_reactive(Vec::new());
        let supplied = sheets.take_reactive();
        assert_eq!(
            frame(&mut sheets, &supplied),
            vec![author("a")],
            "the author sheet survives the empty install"
        );
        let empty = Stylesheet::new(CascadeOrigin::GlobalApp, Vec::new());
        assert_eq!(
            frame(&mut sheets, std::slice::from_ref(&empty)),
            vec![empty],
            "a sheet with no rules is what actually clears it"
        );
    }

    #[test]
    fn taking_the_reactive_set_leaves_none_behind_and_installing_puts_it_back() {
        let mut sheets = CascadeSheets::new();
        sheets.set_reactive(vec![author("a")]);
        let taken = sheets.take_reactive();
        assert_eq!(taken, vec![author("a")]);
        assert!(
            sheets.take_reactive().is_empty(),
            "the runtime holds nothing while the frame has them"
        );
        sheets.set_reactive(taken);
        assert_eq!(sheets.take_reactive(), vec![author("a")]);
    }
}
