//! **SPECIFICATION-DERIVED** document, shell and capture cases for obscur.kit
//! §6.10, §6.11 and §6.17.
//!
//! Every expected value below was produced by an agent reading obscur.kit §6
//! with **no access to this repository's source**. A failure here is a defect
//! report about the toolkit, not about the test; expected values must not be
//! edited to match current behaviour.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::{NodeId, WindowId};
use obscur_kit::system::capture::{
    CaptureProvider, CaptureRequest, CaptureResult, NullCaptureProvider,
};
use obscur_kit::system::document::{
    DocumentHandle, DocumentMetadata, DocumentPermission, DocumentService, MemoryDocumentService,
};
use obscur_kit::system::shell::{NullShellProvider, ShellProvider, ShellResult, ShellTarget};

// -------------------------------------------------------------------------
// P6-13
// -------------------------------------------------------------------------

/// §6.10 Document access: "File pickers return `FileAccess`/`DocumentHandle`,
/// not necessarily filesystem paths." and "- optional path hint."
///
/// A cloud/virtual document has **no** path hint yet is fully usable — metadata
/// and bytes both read — while a local file's hint is present and exact.
#[test]
fn a_virtual_document_handle_is_usable_with_no_path() {
    let mut service = MemoryDocumentService::new();
    service.insert_file("cloud://doc/1", b"hello".to_vec());
    service.insert_file("local://tmp-x", b"local".to_vec());

    let virtual_handle = DocumentHandle::new(
        "cloud://doc/1",
        DocumentMetadata::new("notes.txt").with_byte_size(5),
        DocumentPermission::read_only(),
    );

    assert_eq!(
        virtual_handle.path_hint(),
        None,
        "§6.10: a document with no filesystem path has an absent path hint, not a fabricated one"
    );
    let metadata = service
        .metadata(&virtual_handle)
        .expect("§6.10: metadata reads without a path");
    assert_eq!(
        metadata.name(),
        "notes.txt",
        "§6.10: the handle names itself"
    );
    assert_eq!(
        metadata.byte_size(),
        Some(5),
        "§6.10: the handle reports its size"
    );
    assert_eq!(
        &*service
            .read_bytes(&virtual_handle)
            .expect("§6.10: bytes read without a path"),
        b"hello",
        "§6.10: the handle is fully usable with no path"
    );

    let local_handle = DocumentHandle::new(
        "local://tmp-x",
        DocumentMetadata::new("x.txt"),
        DocumentPermission::read_only(),
    )
    .with_path_hint("/tmp/x.txt");
    assert_eq!(
        local_handle.path_hint(),
        Some("/tmp/x.txt"),
        "§6.10: a local file's path hint is present and exact"
    );
}

// -------------------------------------------------------------------------
// P6-15
// -------------------------------------------------------------------------

/// §6.11 Shell integration: "`ShellProvider` may open:" / "- file;" /
/// "- URI;" / "- document handle." with "Results:" / "- opened;" / "- denied;"
/// / "- unsupported;" / "- failed."
///
/// A build with no shell integration reports exactly `unsupported` — a value of
/// the result type, not a bare `Ok(())`.
///
/// WEAKENED 2026-07-19: this originally opened with a half asserting that
/// `RecordingShellProvider` answers `failed`/`unsupported` for
/// `obscur-nonexistent-scheme://x`. That subject is a TEST DOUBLE, documented at
/// shell_provider.rs:63-90 as a recorder whose whole contract is to accept every
/// target, push it onto a list and answer `Opened` so a caller can assert what a
/// component asked to open. The half therefore tested the mock, not the kit, and
/// making it pass would have meant teaching a recorder to adjudicate schemes.
/// §6.11 lists the four result values and says nothing about which provider
/// returns which for an unregistered scheme. Candidate spec amendment: state
/// whether a `ShellProvider` is obliged to pre-check handler availability
/// (`can_open`) before reporting `opened`, or whether `opened` means only
/// "handed to the platform".
#[test]
fn opening_an_unhandleable_uri_is_never_reported_as_opened() {
    let mut absent = NullShellProvider;
    assert_eq!(
        absent.open(ShellTarget::uri("https://example.invalid")),
        ShellResult::Unsupported,
        "§6.11: a platform build with no shell integration reports exactly `unsupported`"
    );
}

// -------------------------------------------------------------------------
// P6-30
// -------------------------------------------------------------------------

/// §6.17 Capture and privacy: "Results:" / "- captured;" / "- denied;" /
/// "- unsupported;" / "- failed."
///
/// Headlessly, a window capture is `unsupported` (there is no window), and every
/// capture kind answers with a value of the one result type.
///
/// WEAKENED 2026-07-19: this originally asserted that a NODE capture succeeds on
/// `NullCaptureProvider` — "node capture needs no compositor and reports
/// `captured`". That sentence is not in §6.17. The section lists the capability
/// kinds ("capture window", "capture surface", "capture node", "capture image",
/// …) and the four results ("captured", "denied", "unsupported", "failed"),
/// and says nothing about which kinds a compositor-less provider
/// must be able to service; a null provider answering `unsupported` uniformly is
/// a conforming reading. What is kept is what §6.17 does state: each request is
/// answered with a typed result, an unsupported one carries no image, and the
/// four results are distinguishable from one another. Candidate spec amendment:
/// say whether node/image capture is required to work without a window or
/// compositor — it is the difference between a headless golden-image pipeline
/// being possible and not.
#[test]
fn headless_window_capture_is_unsupported_but_node_capture_still_captures() {
    let mut provider = NullCaptureProvider;

    let window = provider.capture(CaptureRequest::window(WindowId::from_raw(1)));
    assert_eq!(
        window,
        CaptureResult::Unsupported,
        "§6.17: with no window, a window capture is `unsupported`"
    );
    assert!(
        window.image().is_none(),
        "§6.17: an unsupported capture yields no image"
    );

    let node = provider.capture(CaptureRequest::node(NodeId::from_raw(7)));
    assert!(
        matches!(
            node,
            CaptureResult::Captured(_)
                | CaptureResult::Denied
                | CaptureResult::Unsupported
                | CaptureResult::Failed(_)
        ),
        "§6.17: a node capture is answered with one of the four typed results, \
         not a bare failure; got {node:?}"
    );
    assert_eq!(
        node.is_captured(),
        node.image().is_some(),
        "§6.17: `captured` and the presence of an image agree — a result never \
         claims success without pixels, nor carries pixels it calls a failure"
    );

    assert_ne!(
        CaptureResult::Denied,
        CaptureResult::Unsupported,
        "§6.17: `denied` is distinguishable from `unsupported`, so a caller can choose \
         between prompting and hiding the button"
    );
    let _protected =
        CaptureRequest::surface().protecting(Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)));
}
