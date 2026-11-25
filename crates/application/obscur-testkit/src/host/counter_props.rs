use obscur_view::{
    component::{FromProps, PropsError, decode_required},
    view::ViewProperty,
};

use super::CounterComponent;

#[doc = include_str!("../../docs/api/host/counter_component.doc/CounterProps.md")]
pub struct CounterProps {
    /// The decoded `step`, required — a `step` that is missing or not an integer is
    /// the failure the boundary tests are built on.
    pub step: u32,
}

impl FromProps for CounterProps {
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        Ok(Self {
            step: decode_required(props, CounterComponent::STEP, "step")?,
        })
    }
}
