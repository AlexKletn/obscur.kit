#![doc = include_str!("../../docs/api/provider/web_shell_provider.doc/_module.md")]

use obscur_system::shell::{ShellProvider, ShellResult, ShellTarget};

#[doc = include_str!("../../docs/api/provider/web_shell_provider.doc/WebShellProvider.md")]
#[derive(Debug, Clone, Default)]
pub struct WebShellProvider;

impl WebShellProvider {
    /// Creates a web shell provider (spec §6.11).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ShellProvider for WebShellProvider {
    fn open(&mut self, target: ShellTarget) -> ShellResult {
        #[cfg(target_arch = "wasm32")]
        {
            let url = target_url(&target);
            // Pop-ups may be blocked by the browser unless this call is made in
            // direct response to a user gesture; a blocked open yields no window.
            match web_sys::window().and_then(|w| w.open_with_url(&url).ok().flatten()) {
                Some(_window) => ShellResult::Opened,
                None => ShellResult::Failed("window.open blocked".into()),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = target;
            // No browser on a non-wasm target: honestly recoverable-unsupported.
            ShellResult::Unsupported
        }
    }

    fn can_open(&self, target: &ShellTarget) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = target;
            // Openable whenever a browser window is present to receive the request.
            web_sys::window().is_some()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = target;
            false
        }
    }
}

#[doc = include_str!("../../docs/api/provider/web_shell_provider.doc/target_url.md")]
#[cfg(target_arch = "wasm32")]
fn target_url(target: &ShellTarget) -> String {
    match target {
        ShellTarget::Uri(uri) => uri.to_string(),
        ShellTarget::Document(token) => token.to_string(),
        ShellTarget::File(handle) => handle
            .path_hint()
            .unwrap_or_else(|| handle.token())
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_is_unsupported_on_host_target() {
        let mut provider = WebShellProvider::new();
        let target = ShellTarget::uri("https://example.com");
        assert!(!provider.can_open(&target));
        assert_eq!(provider.open(target), ShellResult::Unsupported);
    }

    #[test]
    fn default_matches_new() {
        let _default = WebShellProvider::default();
        let _new = WebShellProvider::new();
    }
}
