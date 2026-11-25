The desktop's appearance, as CSS custom properties.

One direction of travel: an
[`AppearanceProvider`](obscur_system::appearance::AppearanceProvider) reads the
desktop, [`ThemeVars`] turns what it said into custom properties, and a host
installs them as a sheet under its own. Nothing here reads anything — the
platform work is an adapter's, and this is the translation.
