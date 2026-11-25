//! CLI-level validation of compiled UI IR documents.

use std::{fs, process::Command};

use obscur_ui_ir::{
    codec::UiIrEncoder,
    document::{UiIrDocument, UiIrEvent, UiIrNode},
};
use serde_json::Value;

#[test]
fn reports_ui_structure_and_rejects_duplicate_node_ids() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("view.okir");
    let inspect = |document| {
        fs::write(&path, UiIrEncoder::new().encode(&document)).unwrap();
        Command::new(env!("CARGO_BIN_EXE_obscur-plugin-cli"))
            .arg("inspect-ui")
            .arg(&path)
            .output()
            .unwrap()
    };
    let result = inspect(UiIrDocument::new(
        UiIrNode::element(1, "button")
            .with_event(UiIrEvent::new("click", "confirm"))
            .with_child(UiIrNode::text(2, "Confirm")),
    ));
    assert!(result.status.success(), "{:?}", result.stderr);
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["node_count"], 2);
    assert_eq!(report["root"]["events"][0]["handler"], "confirm");
    assert_eq!(report["root"]["children"][0]["content"], "Confirm");

    let invalid = UiIrNode::fragment(1).with_child(UiIrNode::text(1, "duplicate"));
    let result = inspect(UiIrDocument::new(invalid));
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("DuplicateNodeId"));
}
