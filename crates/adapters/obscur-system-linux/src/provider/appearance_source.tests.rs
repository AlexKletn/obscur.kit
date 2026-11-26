use super::AppearanceSource::{Auto, Gtk, Portal, Qt};

#[test]
fn auto_reads_only_the_active_desktop_settings() {
    for desktop in ["GNOME", "ubuntu:GNOME", " GNOME "] {
        assert_eq!(Auto.session_order(desktop), &[Portal, Gtk]);
    }
    for desktop in ["KDE", "plasma", "custom:KDE"] {
        assert_eq!(Auto.session_order(desktop), &[Portal, Qt]);
    }
    for desktop in ["", "custom", "LXQt", "XFCE", "Hyprland"] {
        assert_eq!(Auto.session_order(desktop), &[Portal]);
    }
}

#[test]
fn explicit_sources_ignore_the_session() {
    for source in [Gtk, Portal, Qt] {
        assert_eq!(source.session_order("GNOME"), &[source]);
        assert_eq!(source.session_order("KDE"), &[source]);
    }
}
