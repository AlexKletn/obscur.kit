use std::process::Command;
use std::sync::OnceLock;

const LOAD_STYLESHEET: &str = r#"
imports.gi.versions.Adw = '1';
const {Adw, Gio} = imports.gi;
Adw.get_major_version();
const data = Gio.resources_lookup_data(
    '/org/gnome/Adwaita/styles/gtk.css', Gio.ResourceLookupFlags.NONE);
print(new TextDecoder().decode(data.get_data()));
"#;

pub(super) fn load() -> Option<&'static str> {
    static CSS: OnceLock<Option<String>> = OnceLock::new();
    CSS.get_or_init(|| {
        let output = Command::new("gjs")
            .args(["-c", LOAD_STYLESHEET])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8(output.stdout).ok()
    })
    .as_deref()
}
