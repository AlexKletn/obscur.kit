mod plugin_ui_event;
mod ui_snapshot;
mod wire_ui;
mod wire_view;

use plugin_ui_event::PluginUiEvent;
use ui_snapshot::UiSnapshot;
use wire_ui::WireUi;

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use obscur_identity::InternedSymbol;
use obscur_okui_compiler::bridge::UiIrViewBridge;
use obscur_ui_ir::{
    codec::UiIrDecoder,
    document::{UiIrExpression, UiIrNode, UiIrNodeKind, UiIrValue},
    validation::UiIrValidator,
};
use serde::Serialize;

use super::{ContributionId, ExtensionPoint, PluginApiError, PluginRegistration, ViewContribution};

/// Validated guest views and their host-assigned event bindings.
/// Clones share the current snapshot, including updates to staged views.
#[derive(Clone)]
pub struct PluginUi {
    snapshot: Rc<RefCell<UiSnapshot>>,
    max_bytes: usize,
    max_nodes: usize,
}

impl PluginUi {
    /// Decodes version-1 initialization UI using a total node budget.
    ///
    /// The allocator must supply fresh symbols reserved by the host for this
    /// instance. Guest action names never become runtime command symbols.
    pub fn decode(
        bytes: &[u8],
        max_bytes: usize,
        max_nodes: usize,
        allocate_command: impl FnMut() -> InternedSymbol,
    ) -> Result<Self, PluginApiError> {
        let wire = decode_envelope(bytes, max_bytes)?;
        let revision = wire.revision.unwrap_or(0);
        let snapshot = build_snapshot(
            wire,
            revision,
            max_nodes,
            &BTreeMap::new(),
            allocate_command,
        )?;
        Ok(Self {
            snapshot: Rc::new(RefCell::new(snapshot)),
            max_bytes,
            max_nodes,
        })
    }

    /// Returns the currently applied guest revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.snapshot.borrow().revision
    }

    /// Applies a newer full snapshot to the existing staged views.
    ///
    /// Returns false for a stale or duplicate revision. A malformed or
    /// incompatible update leaves both views and event bindings intact.
    pub fn apply_update(
        &self,
        bytes: &[u8],
        allocate_command: impl FnMut() -> InternedSymbol,
    ) -> Result<bool, PluginApiError> {
        let wire = decode_envelope(bytes, self.max_bytes)?;
        let revision = wire
            .revision
            .ok_or_else(|| reject("plugin UI update has no revision"))?;
        let current = self.snapshot.borrow();
        if revision <= current.revision {
            return Ok(false);
        }
        let base_revision = current.revision;
        let previous_events = current.events.clone();
        drop(current);

        let next = build_snapshot(
            wire,
            revision,
            self.max_nodes,
            &previous_events,
            allocate_command,
        )?;
        let mut current = self.snapshot.borrow_mut();
        if current.revision != base_revision {
            return Err(reject("plugin UI changed while preparing an update"));
        }
        if !current
            .views
            .iter()
            .map(|(id, (definition, _))| (id, definition))
            .eq(next
                .views
                .iter()
                .map(|(id, (definition, _))| (id, definition)))
        {
            return Err(reject(
                "plugin UI update changes the registered contributions",
            ));
        }
        *current = next;
        Ok(true)
    }

    /// Stages the validated views using the application's integration points.
    pub fn stage(&self, registration: &mut PluginRegistration) {
        for ((point, id), (definition, _)) in &self.snapshot.borrow().views {
            let key = (point.clone(), id.clone());
            let snapshot = Rc::clone(&self.snapshot);
            registration.contribute_view(point.clone(), definition.clone(), move || {
                snapshot.borrow().views[&key].1.clone()
            });
        }
    }

    /// Iterates the current command symbols to bind in the application's runtime.
    pub fn commands(&self) -> std::vec::IntoIter<InternedSymbol> {
        self.snapshot
            .borrow()
            .events
            .keys()
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Encodes the guest event for one host runtime command, if known.
    pub fn event_message(
        &self,
        command: InternedSymbol,
    ) -> Result<Option<Vec<u8>>, PluginApiError> {
        #[derive(Serialize)]
        struct Message<'a> {
            version: u32,
            #[serde(rename = "type")]
            kind: &'static str,
            event: &'a PluginUiEvent,
        }
        self.snapshot
            .borrow()
            .events
            .get(&command)
            .map(|event| {
                serde_json::to_vec(&Message {
                    version: 1,
                    kind: "ui_event",
                    event,
                })
                .map_err(|_| reject("plugin UI event could not be encoded"))
            })
            .transpose()
    }
}

fn decode_envelope(bytes: &[u8], max_bytes: usize) -> Result<WireUi, PluginApiError> {
    if bytes.len() > max_bytes {
        return Err(reject("plugin UI byte budget exceeded"));
    }
    let wire: WireUi =
        serde_json::from_slice(bytes).map_err(|_| reject("invalid plugin UI envelope"))?;
    if wire.version != 1 {
        return Err(reject("unsupported plugin UI envelope version"));
    }
    Ok(wire)
}

fn build_snapshot(
    wire: WireUi,
    revision: u64,
    max_nodes: usize,
    previous_events: &BTreeMap<InternedSymbol, PluginUiEvent>,
    mut allocate_command: impl FnMut() -> InternedSymbol,
) -> Result<UiSnapshot, PluginApiError> {
    let previous_symbols: BTreeMap<_, _> = previous_events
        .iter()
        .map(|(symbol, event)| (event, *symbol))
        .collect();
    let mut views = BTreeMap::new();
    let mut events = BTreeMap::new();
    let mut remaining = max_nodes;
    for view in wire.views {
        let point =
            ExtensionPoint::new(&view.point).ok_or_else(|| reject("invalid plugin UI point"))?;
        let id = ContributionId::new(&view.id)
            .ok_or_else(|| reject("invalid plugin UI contribution ID"))?;
        let key = (point, id.clone());
        if views.contains_key(&key) {
            return Err(reject("duplicate plugin UI contribution"));
        }
        let document = UiIrDecoder::new()
            .decode_bounded(&view.ir, remaining)
            .map_err(|error| reject(error.to_string()))?;
        remaining -= document.root().node_count();
        if !document.required_capabilities().is_empty()
            || !UiIrValidator::new().validate(&document).is_valid()
        {
            return Err(reject("invalid plugin UI document"));
        }
        validate_node(document.root())?;

        let mut has_collision = false;
        let rendered = UiIrViewBridge::document_with_commands(&document, |node, binding| {
            let event = PluginUiEvent {
                point: view.point.clone(),
                view: view.id.clone(),
                node: node.id(),
                event: binding.event().to_owned(),
                action: binding.handler().to_owned(),
            };
            let symbol = previous_symbols
                .get(&event)
                .copied()
                .unwrap_or_else(&mut allocate_command);
            has_collision |= previous_events
                .get(&symbol)
                .is_some_and(|previous| previous != &event);
            has_collision |= events.insert(symbol, event).is_some();
            symbol
        });
        if has_collision {
            return Err(reject("host reused a plugin UI command symbol"));
        }
        let mut definition = ViewContribution::new(id, view.title).with_order(view.order);
        if let Some(contract) = view.contract {
            definition = definition.with_contract(contract);
        }
        views.insert(key, (definition, rendered));
    }
    Ok(UiSnapshot {
        revision,
        views,
        events,
    })
}

fn validate_node(node: &UiIrNode) -> Result<(), PluginApiError> {
    // A deliberately small static profile. Host component factories and
    // arbitrary property values require a separate authorization contract.
    let supported = match node.kind() {
        UiIrNodeKind::Fragment | UiIrNodeKind::Text => node.events().is_empty(),
        UiIrNodeKind::Element => match node.tag() {
            "div" | "span" => node.events().is_empty(),
            "button" => {
                node.events().len() <= 1
                    && node
                        .events()
                        .iter()
                        .all(|event| event.event() == "click" && !event.handler().is_empty())
            }
            _ => false,
        },
        _ => false,
    };
    if !supported
        || !node.properties().iter().all(|property| {
            node.kind() == UiIrNodeKind::Element
                && matches!(property.name(), "class")
                && matches!(
                    property.value(),
                    UiIrExpression::Literal(UiIrValue::Text(_))
                )
        })
    {
        return Err(reject("unsupported plugin UI node, property, or event"));
    }
    for child in node.children() {
        validate_node(child)?;
    }
    Ok(())
}

fn reject(reason: impl Into<Box<str>>) -> PluginApiError {
    PluginApiError::Rejected(reason.into())
}
