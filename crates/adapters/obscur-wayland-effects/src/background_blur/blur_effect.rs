use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1;
use wayland_protocols_plasma::blur::client::{
    org_kde_kwin_blur::OrgKdeKwinBlur, org_kde_kwin_blur_manager::OrgKdeKwinBlurManager,
};

pub(super) enum BlurEffect {
    Standard(ExtBackgroundEffectSurfaceV1),
    Kde {
        manager: OrgKdeKwinBlurManager,
        surface: WlSurface,
        effect: Option<OrgKdeKwinBlur>,
    },
}
