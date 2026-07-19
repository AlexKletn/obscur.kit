//! Selector matching against a [`MatchElement`]: [`SelectorMatcher`].

use super::complex_selector::{Combinator, ComplexSelector};
use super::compound_selector::CompoundSelector;
use super::match_element::MatchElement;
use super::pseudo_class_selector::{AnB, PseudoClassSelector};
use super::pseudo_state::PseudoState;
use super::selector_list::SelectorList;
use super::selector_specificity::SelectorSpecificity;
use super::simple_selector::{AttrMatch, AttributeSelector, SimpleSelector};

/// The stateless engine that tests selectors against a [`MatchElement`].
///
/// Matching a complex selector is done **right-to-left**: the subject (rightmost)
/// compound is tested against the element, then each combinator to its left is
/// walked against parents or preceding siblings. Descendant and
/// subsequent-sibling combinators backtrack — they try every candidate
/// ancestor / preceding sibling and succeed if any lets the remaining prefix
/// match — while child and next-sibling combinators consider only the single
/// immediate parent / previous sibling.
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectorMatcher;

impl SelectorMatcher {
    /// Returns `true` if `element` matches `selector`.
    ///
    /// Evaluation starts at the subject compound and walks combinators leftward
    /// with backtracking for descendant / subsequent-sibling combinators.
    #[must_use]
    pub fn matches_complex<E: MatchElement>(selector: &ComplexSelector, element: &E) -> bool {
        matches_at(selector, selector.tail().len(), element)
    }

    /// Returns `true` if `element` satisfies every simple selector in
    /// `compound`.
    ///
    /// Type selectors compare local names ASCII case-insensitively; ids and
    /// classes compare exactly. Attribute selectors require the attribute to be
    /// present and, when an operator is given, to satisfy it. Pseudo-elements
    /// never match: the kit generates no box for them, so a compound containing
    /// one is inert (see [`SimpleSelector::PseudoElement`]). Structural
    /// pseudo-classes consult the element's position; state pseudo-classes
    /// consult its [`PseudoState`]; unmodeled pseudo-classes never match.
    #[must_use]
    pub fn matches_compound<E: MatchElement>(compound: &CompoundSelector, element: &E) -> bool {
        compound
            .simples()
            .iter()
            .all(|simple| matches_simple(simple, element))
    }

    /// Returns the highest specificity among the complex selectors in `list`
    /// that match `element`, or [`None`] if none match.
    ///
    /// This is the value the cascade consumes to resolve competing rules.
    #[must_use]
    pub fn matches_list<E: MatchElement>(
        list: &SelectorList,
        element: &E,
    ) -> Option<SelectorSpecificity> {
        list.selectors()
            .iter()
            .filter(|selector| Self::matches_complex(selector, element))
            .map(ComplexSelector::specificity)
            .max()
    }
}

/// Matches the compound at index `k` of `selector` against `element`, then
/// recurses left through the preceding combinator. `k == 0` is the head.
fn matches_at<E: MatchElement>(selector: &ComplexSelector, k: usize, element: &E) -> bool {
    let compound = compound_at(selector, k);
    if !SelectorMatcher::matches_compound(compound, element) {
        return false;
    }
    if k == 0 {
        return true;
    }
    let combinator = selector.tail()[k - 1].0;
    match combinator {
        Combinator::Child => element
            .parent()
            .is_some_and(|parent| matches_at(selector, k - 1, &parent)),
        Combinator::Descendant => {
            let mut ancestor = element.parent();
            while let Some(current) = ancestor {
                if matches_at(selector, k - 1, &current) {
                    return true;
                }
                ancestor = current.parent();
            }
            false
        }
        Combinator::NextSibling => element
            .previous_sibling_element()
            .is_some_and(|prev| matches_at(selector, k - 1, &prev)),
        Combinator::SubsequentSibling => {
            let mut sibling = element.previous_sibling_element();
            while let Some(current) = sibling {
                if matches_at(selector, k - 1, &current) {
                    return true;
                }
                sibling = current.previous_sibling_element();
            }
            false
        }
    }
}

/// Returns the compound at index `k`: `0` is the head, `k` is `tail[k - 1]`.
fn compound_at(selector: &ComplexSelector, k: usize) -> &CompoundSelector {
    if k == 0 {
        selector.head()
    } else {
        &selector.tail()[k - 1].1
    }
}

/// Tests a single simple selector against `element`.
fn matches_simple<E: MatchElement>(simple: &SimpleSelector, element: &E) -> bool {
    match simple {
        SimpleSelector::Type(name) => element
            .local_name()
            .is_some_and(|local| local.eq_ignore_ascii_case(name)),
        SimpleSelector::Universal => true,
        SimpleSelector::Class(class) => element.has_class(class),
        SimpleSelector::Id(id) => element.id() == Some(&**id),
        SimpleSelector::Attribute(attr) => matches_attribute(attr, element),
        // A pseudo-element selects a *generated box* that the kit does not
        // create: there is no `content` property and no synthetic node for it to
        // style. Matching the originating element instead would paint the wrong
        // box (`.badge::before { color: red }` reddening `.badge` itself), so
        // this fails closed and matches nothing. `obscur-css` reports the
        // selector as an unsupported feature, mirroring how the excluded
        // dynamic-state pseudo-classes are both inert and diagnosed (§4.13).
        SimpleSelector::PseudoElement(_) => false,
        SimpleSelector::PseudoClass(pseudo) => matches_pseudo_class(pseudo, element),
    }
}

/// Tests an attribute selector against `element`.
fn matches_attribute<E: MatchElement>(attr: &AttributeSelector, element: &E) -> bool {
    let Some(value) = element.attribute(&attr.name) else {
        return false;
    };
    match &attr.matcher {
        None => true,
        Some(AttrMatch::Exact(operand)) => value == &**operand,
        Some(AttrMatch::Includes(operand)) => {
            !operand.is_empty()
                && value
                    .split_ascii_whitespace()
                    .any(|token| token == &**operand)
        }
        Some(AttrMatch::DashMatch(operand)) => {
            value == &**operand
                || (value.len() > operand.len()
                    && value.starts_with(&**operand)
                    && value.as_bytes()[operand.len()] == b'-')
        }
        Some(AttrMatch::Prefix(operand)) => !operand.is_empty() && value.starts_with(&**operand),
        Some(AttrMatch::Suffix(operand)) => !operand.is_empty() && value.ends_with(&**operand),
        Some(AttrMatch::Substring(operand)) => !operand.is_empty() && value.contains(&**operand),
    }
}

/// Tests a pseudo-class against `element`.
fn matches_pseudo_class<E: MatchElement>(pseudo: &PseudoClassSelector, element: &E) -> bool {
    match pseudo {
        PseudoClassSelector::Hover => element.pseudo_state().contains(PseudoState::HOVER),
        PseudoClassSelector::Focus => element.pseudo_state().contains(PseudoState::FOCUS),
        PseudoClassSelector::FocusVisible => {
            element.pseudo_state().contains(PseudoState::FOCUS_VISIBLE)
        }
        PseudoClassSelector::FocusWithin => {
            element.pseudo_state().contains(PseudoState::FOCUS_WITHIN)
        }
        PseudoClassSelector::Active => element.pseudo_state().contains(PseudoState::ACTIVE),
        PseudoClassSelector::Disabled => element.pseudo_state().contains(PseudoState::DISABLED),
        PseudoClassSelector::Checked => element.pseudo_state().contains(PseudoState::CHECKED),
        // `:enabled` is the negation of `:disabled`.
        PseudoClassSelector::Enabled => !element.pseudo_state().contains(PseudoState::DISABLED),
        PseudoClassSelector::FirstChild => element.index_among_siblings() == 0,
        PseudoClassSelector::LastChild => {
            element.index_among_siblings() + 1 == element.sibling_count()
        }
        PseudoClassSelector::OnlyChild => element.is_only_child(),
        PseudoClassSelector::Root => element.is_root(),
        PseudoClassSelector::Empty => !element.has_children(),
        PseudoClassSelector::NthChild(anb) => {
            anb.matches_index(one_based(element.index_among_siblings() + 1))
        }
        PseudoClassSelector::NthLastChild(anb) => {
            let from_back = element.sibling_count() - element.index_among_siblings();
            anb.matches_index(one_based(from_back))
        }
        PseudoClassSelector::NthOfType(anb) => {
            anb.matches_index(one_based(element.nth_of_type_index()))
        }
        PseudoClassSelector::NthLastOfType(anb) => {
            anb.matches_index(one_based(element.nth_last_of_type_index()))
        }
        PseudoClassSelector::NthChildOf(anb, list) => matches_nth_child_of(*anb, list, element),
        // `:is()` and `:where()` match identically; they differ only in
        // specificity (selectors-4 §4.4: `:where()` has "the same syntax and
        // functionality as :is()").
        PseudoClassSelector::Is(list) | PseudoClassSelector::Where(list) => {
            matches_any(list, element)
        }
        PseudoClassSelector::Not(list) => !matches_any(list, element),
        // `:has()` is decided by looking *forward* from the subject — into its
        // descendants and following siblings — and `MatchElement` exposes no
        // such navigation (only `parent` and `previous_sibling_element`). Adding
        // it is not a local change: `obscur-runtime`'s `RestyleScope` derives its
        // invalidation bound *from* that navigation surface, so a downward-
        // reaching selector would silently under-invalidate. Until both move
        // together this fails closed, exactly as an unmodelled pseudo-class
        // does, and `obscur-css` reports it as unsupported.
        PseudoClassSelector::Has(_) | PseudoClassSelector::Other(_) => false,
    }
}

/// Returns `true` if `element` matches at least one complex selector in `list`.
fn matches_any<E: MatchElement>(list: &SelectorList, element: &E) -> bool {
    list.selectors()
        .iter()
        .any(|selector| SelectorMatcher::matches_complex(selector, element))
}

/// Decides `:nth-child(An+B of S)` (selectors-4 §13.3.1): the element must match
/// `S`, and its index is taken over the sibling list **filtered by `S` first**.
///
/// The filtered index is computed by walking preceding element siblings — the
/// only backward navigation `MatchElement` offers — and counting those that
/// match `S`; the element's own 1-based position in the filtered list is that
/// count plus one.
fn matches_nth_child_of<E: MatchElement>(anb: AnB, list: &SelectorList, element: &E) -> bool {
    if !matches_any(list, element) {
        return false;
    }
    let mut index: usize = 1;
    let mut sibling = element.previous_sibling_element();
    while let Some(current) = sibling {
        if matches_any(list, &current) {
            index += 1;
        }
        sibling = current.previous_sibling_element();
    }
    anb.matches_index(one_based(index))
}

/// Converts a bounded 1-based `usize` index into the `i32` domain [`AnB`] uses,
/// saturating rather than wrapping on the (practically impossible) overflow.
fn one_based(index: usize) -> i32 {
    i32::try_from(index).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A single node in the test arena.
    struct Node {
        local_name: Option<String>,
        id: Option<String>,
        classes: Vec<String>,
        attrs: Vec<(String, String)>,
        state: PseudoState,
        parent: Option<usize>,
        children: Vec<usize>,
    }

    impl Node {
        fn new(local_name: &str) -> Self {
            Self {
                local_name: Some(local_name.to_owned()),
                id: None,
                classes: Vec::new(),
                attrs: Vec::new(),
                state: PseudoState::empty(),
                parent: None,
                children: Vec::new(),
            }
        }
    }

    /// A tree of [`Node`]s addressed by index.
    #[derive(Default)]
    struct Arena {
        nodes: Vec<Node>,
    }

    impl Arena {
        fn add(&mut self, node: Node, parent: Option<usize>) -> usize {
            let idx = self.nodes.len();
            self.nodes.push(node);
            self.nodes[idx].parent = parent;
            if let Some(p) = parent {
                self.nodes[p].children.push(idx);
            }
            idx
        }

        fn get(&self, idx: usize) -> Elem<'_> {
            Elem { arena: self, idx }
        }
    }

    /// A cheap handle implementing [`MatchElement`] over an [`Arena`].
    #[derive(Clone, Copy)]
    struct Elem<'a> {
        arena: &'a Arena,
        idx: usize,
    }

    impl Elem<'_> {
        fn node(&self) -> &Node {
            &self.arena.nodes[self.idx]
        }

        fn siblings(&self) -> Vec<usize> {
            match self.node().parent {
                Some(p) => self.arena.nodes[p].children.clone(),
                None => vec![self.idx],
            }
        }
    }

    impl MatchElement for Elem<'_> {
        fn local_name(&self) -> Option<&str> {
            self.node().local_name.as_deref()
        }

        fn id(&self) -> Option<&str> {
            self.node().id.as_deref()
        }

        fn has_class(&self, class: &str) -> bool {
            self.node().classes.iter().any(|c| c == class)
        }

        fn attribute(&self, name: &str) -> Option<&str> {
            self.node()
                .attrs
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.as_str())
        }

        fn pseudo_state(&self) -> PseudoState {
            self.node().state
        }

        fn parent(&self) -> Option<Self> {
            self.node().parent.map(|p| self.arena.get(p))
        }

        fn previous_sibling_element(&self) -> Option<Self> {
            let siblings = self.siblings();
            let pos = siblings.iter().position(|&i| i == self.idx)?;
            pos.checked_sub(1)
                .map(|prev| self.arena.get(siblings[prev]))
        }

        fn index_among_siblings(&self) -> usize {
            let siblings = self.siblings();
            siblings.iter().position(|&i| i == self.idx).unwrap_or(0)
        }

        fn sibling_count(&self) -> usize {
            self.siblings().len()
        }

        fn is_root(&self) -> bool {
            self.node().parent.is_none()
        }

        fn has_children(&self) -> bool {
            !self.node().children.is_empty()
        }

        // Distinguish types so `:nth-of-type` is genuinely exercised.
        fn nth_of_type_index(&self) -> usize {
            let name = self.node().local_name.clone();
            let siblings = self.siblings();
            let mut count = 0;
            for &sib in &siblings {
                if self.arena.nodes[sib].local_name == name {
                    count += 1;
                }
                if sib == self.idx {
                    break;
                }
            }
            count
        }

        fn nth_last_of_type_index(&self) -> usize {
            let name = self.node().local_name.clone();
            let siblings = self.siblings();
            let mut count = 0;
            for &sib in siblings.iter().rev() {
                if self.arena.nodes[sib].local_name == name {
                    count += 1;
                }
                if sib == self.idx {
                    break;
                }
            }
            count
        }
    }

    // ---- selector-construction helpers -------------------------------------

    fn type_sel(name: &str) -> SimpleSelector {
        SimpleSelector::Type(name.into())
    }

    fn class_sel(name: &str) -> SimpleSelector {
        SimpleSelector::Class(name.into())
    }

    fn id_sel(name: &str) -> SimpleSelector {
        SimpleSelector::Id(name.into())
    }

    fn compound(simples: Vec<SimpleSelector>) -> CompoundSelector {
        CompoundSelector::new(simples)
    }

    fn simple_complex(simples: Vec<SimpleSelector>) -> ComplexSelector {
        ComplexSelector::new(compound(simples), Vec::new())
    }

    // ---- simple / compound -------------------------------------------------

    #[test]
    fn type_selector_is_case_insensitive() {
        let mut arena = Arena::default();
        let root = arena.add(Node::new("DIV"), None);
        let el = arena.get(root);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![type_sel("div")]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![type_sel("span")]),
            &el
        ));
    }

    #[test]
    fn universal_matches_anything() {
        let mut arena = Arena::default();
        let root = arena.add(Node::new("div"), None);
        let el = arena.get(root);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::Universal]),
            &el
        ));
    }

    #[test]
    fn pseudo_element_never_matches_its_originating_element() {
        // `.badge::before` must not select `.badge`: the kit generates no box
        // for the pseudo-element, so the rule is inert rather than mis-applied.
        let mut arena = Arena::default();
        let mut node = Node::new("span");
        node.classes = vec!["badge".to_owned()];
        let root = arena.add(node, None);
        let el = arena.get(root);
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![
                class_sel("badge"),
                SimpleSelector::PseudoElement("before".into()),
            ]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::PseudoElement("after".into())]),
            &el
        ));
        // The bare class still matches — only the pseudo-element compound is inert.
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![class_sel("badge")]),
            &el
        ));
    }

    #[test]
    fn class_and_id_match() {
        let mut arena = Arena::default();
        let mut node = Node::new("div");
        node.id = Some("bar".to_owned());
        node.classes = vec!["foo".to_owned()];
        let root = arena.add(node, None);
        let el = arena.get(root);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![class_sel("foo")]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![class_sel("nope")]),
            &el
        ));
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![id_sel("bar")]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![id_sel("baz")]),
            &el
        ));
    }

    #[test]
    fn compound_requires_all_simples() {
        // div.foo#bar
        let mut arena = Arena::default();
        let mut node = Node::new("div");
        node.id = Some("bar".to_owned());
        node.classes = vec!["foo".to_owned()];
        let root = arena.add(node, None);
        let el = arena.get(root);
        let sel = compound(vec![type_sel("div"), class_sel("foo"), id_sel("bar")]);
        assert!(SelectorMatcher::matches_compound(&sel, &el));

        let miss = compound(vec![type_sel("div"), class_sel("foo"), id_sel("other")]);
        assert!(!SelectorMatcher::matches_compound(&miss, &el));
    }

    // ---- attribute matchers ------------------------------------------------

    fn attr(name: &str, matcher: Option<AttrMatch>) -> SimpleSelector {
        SimpleSelector::Attribute(AttributeSelector {
            name: name.into(),
            matcher,
        })
    }

    fn node_with_attr(name: &str, value: &str) -> Arena {
        let mut arena = Arena::default();
        let mut node = Node::new("div");
        node.attrs = vec![(name.to_owned(), value.to_owned())];
        arena.add(node, None);
        arena
    }

    #[test]
    fn attribute_presence() {
        let arena = node_with_attr("data-x", "y");
        let el = arena.get(0);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr("data-x", None)]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr("data-z", None)]),
            &el
        ));
    }

    #[test]
    fn attribute_exact() {
        let arena = node_with_attr("type", "text");
        let el = arena.get(0);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr("type", Some(AttrMatch::Exact("text".into())))]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr("type", Some(AttrMatch::Exact("number".into())))]),
            &el
        ));
    }

    #[test]
    fn attribute_includes() {
        let arena = node_with_attr("class-like", "a b c");
        let el = arena.get(0);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr(
                "class-like",
                Some(AttrMatch::Includes("b".into()))
            )]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr(
                "class-like",
                Some(AttrMatch::Includes("d".into()))
            )]),
            &el
        ));
        // A substring that is not a whole whitespace-separated token misses.
        let arena2 = node_with_attr("x", "abc def");
        let el2 = arena2.get(0);
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr("x", Some(AttrMatch::Includes("ab".into())))]),
            &el2
        ));
    }

    #[test]
    fn attribute_dash_match() {
        let arena = node_with_attr("lang", "en-US");
        let el = arena.get(0);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr("lang", Some(AttrMatch::DashMatch("en".into())))]),
            &el
        ));
        // Exact also matches.
        let arena2 = node_with_attr("lang", "en");
        let el2 = arena2.get(0);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr("lang", Some(AttrMatch::DashMatch("en".into())))]),
            &el2
        ));
        // Prefix without the `-` boundary misses.
        let arena3 = node_with_attr("lang", "english");
        let el3 = arena3.get(0);
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr("lang", Some(AttrMatch::DashMatch("en".into())))]),
            &el3
        ));
    }

    #[test]
    fn attribute_prefix_suffix_substring() {
        let arena = node_with_attr("href", "https://example.com/page");
        let el = arena.get(0);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr(
                "href",
                Some(AttrMatch::Prefix("https://".into()))
            )]),
            &el
        ));
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr("href", Some(AttrMatch::Suffix("/page".into())))]),
            &el
        ));
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![attr(
                "href",
                Some(AttrMatch::Substring("example".into()))
            )]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr("href", Some(AttrMatch::Prefix("ftp".into())))]),
            &el
        ));
    }

    #[test]
    fn attribute_empty_operand_never_matches() {
        let arena = node_with_attr("x", "value");
        let el = arena.get(0);
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr(
                "x",
                Some(AttrMatch::Prefix(String::new().into()))
            )]),
            &el
        ));
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![attr(
                "x",
                Some(AttrMatch::Includes(String::new().into()))
            )]),
            &el
        ));
    }

    // ---- combinators -------------------------------------------------------

    /// Builds `grandparent > parent > child` (all `div`) and returns the arena
    /// plus the child index.
    fn chain() -> (Arena, usize) {
        let mut arena = Arena::default();
        let gp = arena.add(Node::new("section"), None);
        let parent = arena.add(Node::new("div"), Some(gp));
        let child = arena.add(Node::new("span"), Some(parent));
        (arena, child)
    }

    #[test]
    fn descendant_matches_deep() {
        // "section span" should match the deeply-nested span (backtracking over
        // the intermediate div ancestor).
        let (arena, child) = chain();
        let el = arena.get(child);
        let sel = ComplexSelector::new(
            compound(vec![type_sel("section")]),
            vec![(Combinator::Descendant, compound(vec![type_sel("span")]))],
        );
        assert!(SelectorMatcher::matches_complex(&sel, &el));
    }

    #[test]
    fn child_does_not_skip_generations() {
        // "section > span" must NOT match: span's direct parent is a div.
        let (arena, child) = chain();
        let el = arena.get(child);
        let sel = ComplexSelector::new(
            compound(vec![type_sel("section")]),
            vec![(Combinator::Child, compound(vec![type_sel("span")]))],
        );
        assert!(!SelectorMatcher::matches_complex(&sel, &el));

        // "div > span" matches the immediate parent.
        let sel_ok = ComplexSelector::new(
            compound(vec![type_sel("div")]),
            vec![(Combinator::Child, compound(vec![type_sel("span")]))],
        );
        assert!(SelectorMatcher::matches_complex(&sel_ok, &el));
    }

    /// Builds a parent with three children `a`, `b`, `c` and returns the arena
    /// plus the three child indices.
    fn siblings() -> (Arena, [usize; 3]) {
        let mut arena = Arena::default();
        let parent = arena.add(Node::new("ul"), None);
        let a = arena.add(Node::new("a"), Some(parent));
        let b = arena.add(Node::new("b"), Some(parent));
        let c = arena.add(Node::new("c"), Some(parent));
        (arena, [a, b, c])
    }

    #[test]
    fn next_sibling_is_immediate() {
        let (arena, [_, b, c]) = siblings();
        // "a + b": b immediately follows a -> matches.
        let sel = ComplexSelector::new(
            compound(vec![type_sel("a")]),
            vec![(Combinator::NextSibling, compound(vec![type_sel("b")]))],
        );
        assert!(SelectorMatcher::matches_complex(&sel, &arena.get(b)));
        // "a + c": c does not immediately follow a -> no match.
        let sel2 = ComplexSelector::new(
            compound(vec![type_sel("a")]),
            vec![(Combinator::NextSibling, compound(vec![type_sel("c")]))],
        );
        assert!(!SelectorMatcher::matches_complex(&sel2, &arena.get(c)));
    }

    #[test]
    fn subsequent_sibling_backtracks() {
        let (arena, [_, _, c]) = siblings();
        // "a ~ c": a precedes c among siblings (not immediately) -> matches.
        let sel = ComplexSelector::new(
            compound(vec![type_sel("a")]),
            vec![(Combinator::SubsequentSibling, compound(vec![type_sel("c")]))],
        );
        assert!(SelectorMatcher::matches_complex(&sel, &arena.get(c)));
    }

    // ---- state pseudo-classes ---------------------------------------------

    #[test]
    fn hover_and_focus_via_pseudo_state() {
        let mut arena = Arena::default();
        let mut node = Node::new("button");
        node.state = PseudoState::HOVER;
        let idx = arena.add(node, None);
        let el = arena.get(idx);
        let hover = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Hover,
        )]);
        let focus = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Focus,
        )]);
        assert!(SelectorMatcher::matches_compound(&hover, &el));
        assert!(!SelectorMatcher::matches_compound(&focus, &el));
    }

    #[test]
    fn enabled_is_negation_of_disabled() {
        let mut arena = Arena::default();
        let enabled = arena.add(Node::new("input"), None);
        let mut disabled_node = Node::new("input");
        disabled_node.state = PseudoState::DISABLED;
        let disabled = arena.add(disabled_node, None);
        let enabled_sel = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Enabled,
        )]);
        let disabled_sel = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Disabled,
        )]);
        assert!(SelectorMatcher::matches_compound(
            &enabled_sel,
            &arena.get(enabled)
        ));
        assert!(!SelectorMatcher::matches_compound(
            &disabled_sel,
            &arena.get(enabled)
        ));
        assert!(!SelectorMatcher::matches_compound(
            &enabled_sel,
            &arena.get(disabled)
        ));
        assert!(SelectorMatcher::matches_compound(
            &disabled_sel,
            &arena.get(disabled)
        ));
    }

    // ---- structural pseudo-classes ----------------------------------------

    #[test]
    fn first_last_only_child() {
        let (arena, [a, _, c]) = siblings();
        let first = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::FirstChild,
        )]);
        let last = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::LastChild,
        )]);
        assert!(SelectorMatcher::matches_compound(&first, &arena.get(a)));
        assert!(!SelectorMatcher::matches_compound(&first, &arena.get(c)));
        assert!(SelectorMatcher::matches_compound(&last, &arena.get(c)));
        assert!(!SelectorMatcher::matches_compound(&last, &arena.get(a)));

        // Only-child: sole child of a parent.
        let mut solo_arena = Arena::default();
        let parent = solo_arena.add(Node::new("ul"), None);
        let only = solo_arena.add(Node::new("li"), Some(parent));
        let only_sel = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::OnlyChild,
        )]);
        assert!(SelectorMatcher::matches_compound(
            &only_sel,
            &solo_arena.get(only)
        ));
        assert!(!SelectorMatcher::matches_compound(&only_sel, &arena.get(a)));
    }

    #[test]
    fn root_and_empty() {
        let mut arena = Arena::default();
        let root = arena.add(Node::new("html"), None);
        let child = arena.add(Node::new("body"), Some(root));
        let root_sel = compound(vec![SimpleSelector::PseudoClass(PseudoClassSelector::Root)]);
        let empty_sel = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Empty,
        )]);
        assert!(SelectorMatcher::matches_compound(
            &root_sel,
            &arena.get(root)
        ));
        assert!(!SelectorMatcher::matches_compound(
            &root_sel,
            &arena.get(child)
        ));
        // root has a child -> not empty; child has none -> empty.
        assert!(!SelectorMatcher::matches_compound(
            &empty_sel,
            &arena.get(root)
        ));
        assert!(SelectorMatcher::matches_compound(
            &empty_sel,
            &arena.get(child)
        ));
    }

    /// Parent with `n` `li` children; returns arena + child indices.
    fn many_children(n: usize) -> (Arena, Vec<usize>) {
        let mut arena = Arena::default();
        let parent = arena.add(Node::new("ul"), None);
        let kids = (0..n)
            .map(|_| arena.add(Node::new("li"), Some(parent)))
            .collect();
        (arena, kids)
    }

    #[test]
    fn nth_child_odd_via_two_n_plus_one() {
        let (arena, kids) = many_children(5);
        let odd = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::NthChild(AnB::new(2, 1)),
        )]);
        // 1-based positions 1,3,5 (indices 0,2,4) match.
        for (pos, &kid) in kids.iter().enumerate() {
            let expected = pos % 2 == 0;
            assert_eq!(
                SelectorMatcher::matches_compound(&odd, &arena.get(kid)),
                expected,
                "position {}",
                pos + 1
            );
        }
    }

    #[test]
    fn nth_child_constant_selects_one() {
        let (arena, kids) = many_children(5);
        let third = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::NthChild(AnB::new(0, 3)),
        )]);
        for (pos, &kid) in kids.iter().enumerate() {
            assert_eq!(
                SelectorMatcher::matches_compound(&third, &arena.get(kid)),
                pos + 1 == 3
            );
        }
    }

    #[test]
    fn nth_last_child_counts_from_back() {
        let (arena, kids) = many_children(4);
        // :nth-last-child(1) is the last element.
        let last = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::NthLastChild(AnB::new(0, 1)),
        )]);
        assert!(SelectorMatcher::matches_compound(
            &last,
            &arena.get(kids[3])
        ));
        assert!(!SelectorMatcher::matches_compound(
            &last,
            &arena.get(kids[0])
        ));
    }

    #[test]
    fn nth_of_type_distinguishes_types() {
        // Mixed children: a, b, a, b, a. :nth-of-type(2) of type `a` is the 3rd
        // child (2nd `a`).
        let mut arena = Arena::default();
        let parent = arena.add(Node::new("ul"), None);
        let names = ["a", "b", "a", "b", "a"];
        let kids: Vec<usize> = names
            .iter()
            .map(|n| arena.add(Node::new(n), Some(parent)))
            .collect();
        let second_a = compound(vec![
            type_sel("a"),
            SimpleSelector::PseudoClass(PseudoClassSelector::NthOfType(AnB::new(0, 2))),
        ]);
        // 3rd child (index 2) is the 2nd `a`.
        assert!(SelectorMatcher::matches_compound(
            &second_a,
            &arena.get(kids[2])
        ));
        // 1st child (index 0) is the 1st `a`, not the 2nd.
        assert!(!SelectorMatcher::matches_compound(
            &second_a,
            &arena.get(kids[0])
        ));
    }

    #[test]
    fn other_pseudo_class_never_matches() {
        let mut arena = Arena::default();
        let idx = arena.add(Node::new("div"), None);
        let sel = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Other("visited".into()),
        )]);
        assert!(!SelectorMatcher::matches_compound(&sel, &arena.get(idx)));
    }

    // ---- selector list & specificity --------------------------------------

    #[test]
    fn list_returns_highest_specificity() {
        let mut arena = Arena::default();
        let mut node = Node::new("div");
        node.id = Some("main".to_owned());
        node.classes = vec!["box".to_owned()];
        let idx = arena.add(node, None);
        let el = arena.get(idx);

        // List: `div` (0,0,1), `.box` (0,1,0), `#main` (1,0,0). All match; the
        // id selector wins.
        let list = SelectorList::new(vec![
            simple_complex(vec![type_sel("div")]),
            simple_complex(vec![class_sel("box")]),
            simple_complex(vec![id_sel("main")]),
        ]);
        assert_eq!(
            SelectorMatcher::matches_list(&list, &el),
            Some(SelectorSpecificity::new(1, 0, 0))
        );
    }

    #[test]
    fn list_returns_none_when_nothing_matches() {
        let mut arena = Arena::default();
        let idx = arena.add(Node::new("div"), None);
        let list = SelectorList::new(vec![
            simple_complex(vec![type_sel("span")]),
            simple_complex(vec![class_sel("nope")]),
        ]);
        assert_eq!(SelectorMatcher::matches_list(&list, &arena.get(idx)), None);
    }

    #[test]
    fn id_beats_class_specificity() {
        // Ensure the "highest" is truly by specificity, not source order.
        let mut arena = Arena::default();
        let mut node = Node::new("div");
        node.id = Some("x".to_owned());
        node.classes = vec!["y".to_owned()];
        let idx = arena.add(node, None);
        let list = SelectorList::new(vec![
            simple_complex(vec![id_sel("x")]),
            simple_complex(vec![class_sel("y")]),
        ]);
        let spec = SelectorMatcher::matches_list(&list, &arena.get(idx)).unwrap();
        assert_eq!(spec, SelectorSpecificity::new(1, 0, 0));
    }

    // ---- selector-list pseudo-classes --------------------------------------

    /// `ul` with five children: `p`, then four `li.important`.
    fn nth_of_fixture() -> (Arena, Vec<usize>) {
        let mut arena = Arena::default();
        let parent = arena.add(Node::new("ul"), None);
        let mut kids = vec![arena.add(Node::new("p"), Some(parent))];
        for _ in 0..4 {
            let mut node = Node::new("li");
            node.classes = vec!["important".to_owned()];
            kids.push(arena.add(node, Some(parent)));
        }
        (arena, kids)
    }

    #[test]
    fn is_matches_any_argument_and_where_matches_identically() {
        let mut arena = Arena::default();
        let mut node = Node::new("em");
        node.classes = vec!["c".to_owned()];
        let idx = arena.add(node, None);
        let list = SelectorList::new(vec![
            simple_complex(vec![type_sel("em")]),
            simple_complex(vec![id_sel("foo")]),
        ]);
        let el = arena.get(idx);
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::PseudoClass(PseudoClassSelector::Is(
                list.clone()
            ))]),
            &el
        ));
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::PseudoClass(
                PseudoClassSelector::Where(list.clone())
            )]),
            &el
        ));
        // Nothing in the list matches: neither form does either.
        let miss = SelectorList::new(vec![simple_complex(vec![type_sel("span")])]);
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::PseudoClass(PseudoClassSelector::Is(
                miss
            ))]),
            &el
        ));
    }

    #[test]
    fn not_is_the_negation_of_the_whole_list() {
        let mut arena = Arena::default();
        let idx = arena.add(Node::new("div"), None);
        let el = arena.get(idx);
        let hits = SelectorList::new(vec![
            simple_complex(vec![type_sel("em")]),
            simple_complex(vec![type_sel("div")]),
        ]);
        let misses = SelectorList::new(vec![
            simple_complex(vec![type_sel("em")]),
            simple_complex(vec![type_sel("strong")]),
        ]);
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::PseudoClass(PseudoClassSelector::Not(
                hits
            ))]),
            &el
        ));
        assert!(SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::PseudoClass(PseudoClassSelector::Not(
                misses
            ))]),
            &el
        ));
    }

    #[test]
    fn has_never_matches_because_the_seam_cannot_look_forward() {
        let mut arena = Arena::default();
        let parent = arena.add(Node::new("ul"), None);
        let _child = arena.add(Node::new("li"), Some(parent));
        let list = SelectorList::new(vec![simple_complex(vec![type_sel("li")])]);
        assert!(!SelectorMatcher::matches_compound(
            &compound(vec![SimpleSelector::PseudoClass(PseudoClassSelector::Has(
                list
            ))]),
            &arena.get(parent)
        ));
    }

    #[test]
    fn nth_child_of_s_indexes_the_filtered_sibling_list() {
        let (arena, kids) = nth_of_fixture();
        // :nth-child(-n+3 of li.important) — the first three *matching* siblings,
        // which are children 2, 3 and 4 of the parent.
        let s = SelectorList::new(vec![simple_complex(vec![
            type_sel("li"),
            class_sel("important"),
        ])]);
        let sel = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::NthChildOf(AnB::new(-1, 3), s),
        )]);
        let hits: Vec<bool> = kids
            .iter()
            .map(|&k| SelectorMatcher::matches_compound(&sel, &arena.get(k)))
            .collect();
        assert_eq!(hits, vec![false, true, true, true, false]);
    }

    #[test]
    fn nth_child_of_s_differs_from_the_compound_form() {
        let (arena, kids) = nth_of_fixture();
        // li.important:nth-child(-n+3) indexes over *all* siblings, so it stops
        // at child 3 where the `of S` form reaches child 4.
        let sel = compound(vec![
            type_sel("li"),
            class_sel("important"),
            SimpleSelector::PseudoClass(PseudoClassSelector::NthChild(AnB::new(-1, 3))),
        ]);
        let hits: Vec<bool> = kids
            .iter()
            .map(|&k| SelectorMatcher::matches_compound(&sel, &arena.get(k)))
            .collect();
        assert_eq!(hits, vec![false, true, true, false, false]);
    }

    #[test]
    fn nth_child_of_s_requires_the_element_itself_to_match_s() {
        let (arena, kids) = nth_of_fixture();
        // The `p` first child is never selected, whatever its index would be.
        let s = SelectorList::new(vec![simple_complex(vec![class_sel("important")])]);
        let sel = compound(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::NthChildOf(AnB::new(0, 1), s),
        )]);
        assert!(!SelectorMatcher::matches_compound(
            &sel,
            &arena.get(kids[0])
        ));
        assert!(SelectorMatcher::matches_compound(&sel, &arena.get(kids[1])));
    }
}
