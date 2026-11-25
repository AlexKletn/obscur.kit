//! Bounded validation and structural inspection of an OKIR document.

use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use obscur_ui_ir::{codec::UiIrDecoder, document::UiIrNode, validation::UiIrValidator};
use serde_json::{Value, json};

pub(crate) fn run(path: &Path, mut output: impl Write) -> Result<(), Box<dyn Error>> {
    const MAX_BYTES: usize = 4 * 1024 * 1024;
    const MAX_NODES: usize = 16_384;
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err("OKIR document exceeds 4 MiB".into());
    }
    let document = UiIrDecoder::new().decode_bounded(&bytes, MAX_NODES)?;
    let validation = UiIrValidator::new().validate(&document);
    if !validation.is_valid() {
        return Err(format!("invalid OKIR document: {:?}", validation.errors()).into());
    }
    let version = document.version();
    serde_json::to_writer_pretty(
        &mut output,
        &json!({
        "version": {"major": version.major(), "minor": version.minor()},
            "node_count": document.node_count(),
            "required_capabilities": document.required_capabilities().iter().map(|id| id.as_str()).collect::<Vec<_>>(),
            "root": inspect_node(document.root()),
        }),
    )?;
    writeln!(output)?;
    Ok(())
}

fn inspect_node(node: &UiIrNode) -> Value {
    json!({
        "id": node.id(),
        "kind": format!("{:?}", node.kind()),
        "tag": node.tag(),
        "content": node.content(),
        "properties": node.properties().iter().map(|property| property.name()).collect::<Vec<_>>(),
        "events": node.events().iter().map(|event| json!({
            "event": event.event(),
            "handler": event.handler(),
        })).collect::<Vec<_>>(),
        "children": node.children().iter().map(inspect_node).collect::<Vec<_>>(),
    })
}
