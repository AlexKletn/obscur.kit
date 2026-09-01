Appearance as GNOME states it, through `gsettings`.

Fonts, icon theme, cursor size, text scaling, animations, colour scheme, and —
on GNOME 47 and later — an accent.

**It states no colours, and it cannot.** `org.gnome.desktop.interface` names the
*theme*; the theme's palette lives inside GTK and reading it means linking GTK,
which this adapter deliberately does not do (the whole `obscur-system-linux`
adapter is `std::process` and nothing else). So the palette comes back empty and
[`support`](Self::support) answers
[`Limited`](obscur_capability::SupportLevel::Limited) — permanently, not until
someone gets around to it.
