Materializes deferred windows once the platform is ready to create them.

Some platforms (notably Android/Web) forbid window creation before the
first `resumed`; deferring creation until here — and re-pumping on every
`about_to_wait` — is what lets `create_window` return an id synchronously
yet still produce a real window.
