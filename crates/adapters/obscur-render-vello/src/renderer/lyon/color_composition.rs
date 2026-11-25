//! Map OpenType composition modes to the shared layer compositor.

use skrifa::color::CompositeMode;
use vello::peniko::{Compose, Mix};

pub(super) fn blend(mode: CompositeMode) -> Option<u32> {
    use CompositeMode as Colr;

    let (mix, compose) = match mode {
        Colr::SrcOver => return None,
        Colr::Clear => (Mix::Normal, Compose::Clear),
        Colr::Src => (Mix::Normal, Compose::Copy),
        Colr::Dest => (Mix::Normal, Compose::Dest),
        Colr::DestOver => (Mix::Normal, Compose::DestOver),
        Colr::SrcIn => (Mix::Normal, Compose::SrcIn),
        Colr::DestIn => (Mix::Normal, Compose::DestIn),
        Colr::SrcOut => (Mix::Normal, Compose::SrcOut),
        Colr::DestOut => (Mix::Normal, Compose::DestOut),
        Colr::SrcAtop => (Mix::Normal, Compose::SrcAtop),
        Colr::DestAtop => (Mix::Normal, Compose::DestAtop),
        Colr::Xor => (Mix::Normal, Compose::Xor),
        Colr::Plus => (Mix::Normal, Compose::Plus),
        Colr::Screen => (Mix::Screen, Compose::SrcOver),
        Colr::Overlay => (Mix::Overlay, Compose::SrcOver),
        Colr::Darken => (Mix::Darken, Compose::SrcOver),
        Colr::Lighten => (Mix::Lighten, Compose::SrcOver),
        Colr::ColorDodge => (Mix::ColorDodge, Compose::SrcOver),
        Colr::ColorBurn => (Mix::ColorBurn, Compose::SrcOver),
        Colr::HardLight => (Mix::HardLight, Compose::SrcOver),
        Colr::SoftLight => (Mix::SoftLight, Compose::SrcOver),
        Colr::Difference => (Mix::Difference, Compose::SrcOver),
        Colr::Exclusion => (Mix::Exclusion, Compose::SrcOver),
        Colr::Multiply => (Mix::Multiply, Compose::SrcOver),
        Colr::HslHue => (Mix::Hue, Compose::SrcOver),
        Colr::HslSaturation => (Mix::Saturation, Compose::SrcOver),
        Colr::HslColor => (Mix::Color, Compose::SrcOver),
        Colr::HslLuminosity => (Mix::Luminosity, Compose::SrcOver),
        _ => (Mix::Normal, Compose::Clear),
    };
    Some((mix as u32) << 8 | compose as u32)
}
