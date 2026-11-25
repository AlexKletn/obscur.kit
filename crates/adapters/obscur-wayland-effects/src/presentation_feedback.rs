use wayland_client::globals::GlobalListContents;
use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_registry};
use wayland_protocols::wp::presentation_time::client::{
    wp_presentation::{self, WpPresentation},
    wp_presentation_feedback::{self, WpPresentationFeedback},
};

/// Frame outcomes queued by the protocol dispatcher.
#[derive(Default)]
pub(crate) struct PresentationFeedback {
    pub(crate) frames: Vec<(u64, Option<u64>)>,
    pub(crate) clock_id: Option<u32>,
}

impl Dispatch<WpPresentation, ()> for PresentationFeedback {
    fn event(
        state: &mut Self,
        _: &WpPresentation,
        event: wp_presentation::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_presentation::Event::ClockId { clk_id } = event {
            state.clock_id = Some(clk_id);
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for PresentationFeedback {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WpPresentationFeedback, u64> for PresentationFeedback {
    fn event(
        state: &mut Self,
        _: &WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        frame: &u64,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wp_presentation_feedback::Event::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                ..
            } => {
                let micros = crate::presentation_timestamp_micros(tv_sec_hi, tv_sec_lo, tv_nsec);
                state.frames.push((*frame, Some(micros)));
            }
            wp_presentation_feedback::Event::Discarded => state.frames.push((*frame, None)),
            _ => {}
        }
    }
}
