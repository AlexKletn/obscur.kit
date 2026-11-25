use obscur_motion::track::Easing;
use obscur_style::matching::{MediaEnvironment, RuleCondition};
use obscur_style::property::PropertyId;
use obscur_style::selector::SelectorList;

/// A CSS `transition` bound to a selector: on the elements this selector matches,
/// a change to `property`'s computed value is eased over `duration_secs` using
/// `easing` (spec §4.17).
#[derive(Debug, Clone, PartialEq)]
pub struct CssTransitionRule {
    condition: Option<RuleCondition>,
    /// The selector list whose matched elements carry this transition.
    selectors: SelectorList,
    /// The property whose change triggers the transition.
    property: Option<PropertyId>,
    cascade: Option<(usize, bool)>,
    /// The transition duration, in seconds.
    duration_secs: f32,
    /// The easing curve applied over the transition.
    easing: Easing,
    /// `transition-delay`, in seconds. A negative delay makes the transition
    /// start part-way through its play cycle (css-transitions-1 §2.4).
    delay_secs: f32,
}

impl CssTransitionRule {
    /// Creates a transition rule.
    #[must_use]
    pub fn new(
        selectors: SelectorList,
        property: PropertyId,
        duration_secs: f32,
        easing: Easing,
    ) -> Self {
        Self {
            condition: None,
            selectors,
            property: Some(property),
            cascade: None,
            duration_secs,
            easing,
            delay_secs: 0.0,
        }
    }

    /// A transition shorthand that clears the list of transitioning properties.
    #[must_use]
    pub fn reset(selectors: SelectorList) -> Self {
        Self {
            condition: None,
            selectors,
            property: None,
            cascade: None,
            duration_secs: 0.0,
            easing: Easing::Linear,
            delay_secs: 0.0,
        }
    }

    /// Identifies the shorthand declaration and its importance in the cascade.
    #[must_use]
    pub fn with_cascade(mut self, source_order: usize, important: bool) -> Self {
        self.cascade = Some((source_order, important));
        self
    }

    pub(crate) fn cascade(&self) -> Option<(usize, bool)> {
        self.cascade
    }

    /// Restricts this transition rule to matching environments. Passing `None`
    /// makes the rule unconditional.
    #[must_use]
    pub fn with_condition(mut self, condition: Option<RuleCondition>) -> Self {
        self.condition = condition;
        self
    }

    pub(crate) fn applies(&self, environment: &MediaEnvironment<'_>) -> bool {
        self.condition
            .as_ref()
            .is_none_or(|condition| condition.holds(environment))
    }

    #[doc = include_str!("../../docs/api/orchestration/css_motion_rules.doc/with_delay.md")]
    #[must_use]
    pub fn with_delay(mut self, delay_secs: f32) -> Self {
        self.delay_secs = delay_secs;
        self
    }

    /// Returns `transition-delay`, in seconds.
    #[must_use]
    pub fn delay_secs(&self) -> f32 {
        self.delay_secs
    }

    /// Returns the selector list this transition applies to.
    #[must_use]
    pub fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    /// Returns the transitioned property. A reset declaration has no target property.
    #[must_use]
    pub fn property(&self) -> Option<PropertyId> {
        self.property
    }

    /// Returns the transition duration, in seconds.
    #[must_use]
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }

    /// Returns the easing curve.
    #[must_use]
    pub fn easing(&self) -> Easing {
        self.easing
    }
}
