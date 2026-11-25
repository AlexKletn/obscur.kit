use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_compositor, wl_region, wl_registry};
use wayland_client::{Connection, Dispatch, QueueHandle, delegate_noop};
use wayland_protocols::ext::background_effect::v1::client::{
    ext_background_effect_manager_v1::{self as manager, ExtBackgroundEffectManagerV1},
    ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1,
};
use wayland_protocols_plasma::blur::client::{
    org_kde_kwin_blur::OrgKdeKwinBlur, org_kde_kwin_blur_manager::OrgKdeKwinBlurManager,
};

#[derive(Default)]
pub(super) struct BlurDispatch {
    pub(super) blur: bool,
}

impl Dispatch<ExtBackgroundEffectManagerV1, ()> for BlurDispatch {
    fn event(
        state: &mut Self,
        _: &ExtBackgroundEffectManagerV1,
        event: manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let manager::Event::Capabilities { flags } = event {
            state.blur = u32::from(flags) & manager::Capability::Blur.bits() != 0;
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for BlurDispatch {
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

delegate_noop!(BlurDispatch: ignore wl_compositor::WlCompositor);
delegate_noop!(BlurDispatch: ignore wl_region::WlRegion);
delegate_noop!(BlurDispatch: ignore ExtBackgroundEffectSurfaceV1);
delegate_noop!(BlurDispatch: ignore OrgKdeKwinBlurManager);
delegate_noop!(BlurDispatch: ignore OrgKdeKwinBlur);
