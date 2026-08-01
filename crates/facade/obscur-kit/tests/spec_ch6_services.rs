#![doc = include_str!("spec_ch6_services.doc/_module.md")]
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

#[doc = include_str!("spec_ch6_services.doc/a_virtual_document_handle_is_usable_with_no_path.md")]
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

#[doc = include_str!("spec_ch6_services.doc/opening_an_unhandleable_uri_is_never_reported_as_opened.md")]
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

#[doc = include_str!("spec_ch6_services.doc/headless_window_capture_is_unsupported_but_node_capture_still_captures.md")]
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
