//! A web [`ShellProvider`] that opens targets via `window.open` (spec §6.11).
//!
//! [`WebShellProvider`] implements the pure
//! [`ShellProvider`](obscur_system::shell::ShellProvider) port in the browser by
//! calling [`Window::open_with_url`](web_sys::Window::open_with_url). The open is
//! best-effort: a browser may block the pop-up (for example when the call is not
//! made in direct response to a user gesture), in which case `open` returns no
//! window handle and this provider reports [`ShellResult::Failed`].
//!
//! The `window.open` binding only exists on `wasm32`; on any other target the
//! provider reports [`ShellResult::Unsupported`] and [`Self::can_open`] is
//! `false`, so the crate stays honest on the default host build.

use obscur_system::shell::{ShellProvider, ShellResult, ShellTarget};

/// A web [`ShellProvider`] backed by `window.open` (spec §6.11).
///
/// The provider is stateless: it forwards each target to the browser's
/// `window.open`. Because pop-ups may be blocked by the browser unless the call
/// originates from a user gesture, a successful spawn is not guaranteed — a
/// blocked open is reported as [`ShellResult::Failed`].
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

/// Extracts the string handed to `window.open` for `target` (spec §6.11).
///
/// A [`ShellTarget::Uri`] contributes the URI, a [`ShellTarget::Document`] its
/// opaque token, and a [`ShellTarget::File`] its advisory path hint when present,
/// falling back to the handle token otherwise — mirroring the Linux adapter.
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
