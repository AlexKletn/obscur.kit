use super::*;

#[test]
fn snapshot_payload_requires_four_complete_pngs() {
    let mut record = Vec::new();
    record.extend(34u32.to_le_bytes());
    record.extend(32u32.to_le_bytes());
    record.extend(8u32.to_le_bytes());
    record.extend(b"\x89PNG\r\n\x1a\n");
    let complete = record.repeat(4);
    let decoded = decode(&complete).unwrap();
    assert_eq!(decoded.size(), (34, 32));
    assert!(decoded.png("pressed").is_some());
    assert!(decoded.png("other").is_none());
    assert!(decode(&complete[..complete.len() - 1]).is_none());
}
