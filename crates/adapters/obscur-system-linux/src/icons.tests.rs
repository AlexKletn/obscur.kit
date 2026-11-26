use super::*;

#[test]
fn invalid_names_and_dimensions_do_not_resolve() {
    for name in ["../close", "/close", "./close", "close/", "", "a\0b"] {
        assert!(theme_icon_file("Adwaita", name, 16, 1).is_none());
        assert!(theme_icon_file(name, "window-close-symbolic", 16, 1).is_none());
    }
    assert!(theme_icon_file("Adwaita", "window-close-symbolic", 0, 1).is_none());
    assert!(theme_icon_file("Adwaita", "window-close-symbolic", 16, 0).is_none());
}

#[test]
fn absent_theme_and_unsupported_roles_have_no_file() {
    assert!(
        window_button_icon_file(
            &SystemAppearance::default(),
            WindowButton::Close,
            false,
            16,
            1
        )
        .is_none()
    );
    let appearance = SystemAppearance {
        icon_theme: Some("Adwaita".into()),
        ..SystemAppearance::default()
    };
    assert!(window_button_icon_file(&appearance, WindowButton::Spacer, false, 16, 1).is_none());
}

#[test]
#[ignore = "requires the Adwaita icon theme"]
fn adwaita_provides_files_for_maximize_and_restore() {
    let appearance = SystemAppearance {
        icon_theme: Some("Adwaita".into()),
        ..SystemAppearance::default()
    };
    let close = theme_icon_svg("Adwaita", "window-close-symbolic", 16, 1).unwrap();
    let close = std::str::from_utf8(&close).unwrap();
    assert!(close.contains("currentColor"));
    assert!(!close.contains("#2e3436"));
    for (maximized, expected) in [
        (false, "window-maximize-symbolic.svg"),
        (true, "window-restore-symbolic.svg"),
    ] {
        let path = window_button_icon_file(&appearance, WindowButton::Maximize, maximized, 16, 1)
            .expect("Adwaita provides both window icons");
        assert_eq!(path.file_name().unwrap(), expected);
        assert!(std::fs::read(path).unwrap().starts_with(b"<"));
    }
}
