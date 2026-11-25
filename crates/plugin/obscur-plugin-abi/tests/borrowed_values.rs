//! Bounds and ownership checks for borrowed C ABI values.

use obscur_plugin_abi::value::{AbiBuffer, AbiSlice, AbiString};

#[test]
fn rejects_oversized_and_invalid_borrowed_input() {
    let null = AbiSlice {
        data: std::ptr::null(),
        len: 1,
    };
    let bytes = AbiSlice::from_bytes(b"hello");
    let text = AbiString::from_text("hello");
    let invalid_text = AbiString(AbiSlice::from_bytes(&[0xff]));
    // SAFETY: nonnull inputs borrow live arrays or string literals.
    unsafe {
        assert_eq!(null.as_bytes(1), None);
        assert_eq!(bytes.as_bytes(4), None);
        assert_eq!(text.as_str(5), Some("hello"));
        assert_eq!(invalid_text.as_str(1), None);
    }
}

#[test]
fn caller_owned_buffer_never_writes_a_partial_reply() {
    let mut bytes = [0; 4];
    let buffer = AbiBuffer::from_bytes(&mut bytes);
    // SAFETY: the output borrows the live array and the source is disjoint.
    assert!(!unsafe { buffer.write(b"12345") });
    assert_eq!(bytes, [0; 4]);
    let buffer = AbiBuffer::from_bytes(&mut bytes);
    // SAFETY: this call has exclusive access to the output bytes.
    assert!(unsafe { buffer.write(b"123") });
    assert_eq!(bytes, *b"123\0");
}
