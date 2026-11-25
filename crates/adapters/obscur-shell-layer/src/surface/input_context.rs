use obscur_identity::NodeId;

#[derive(PartialEq)]
pub(super) struct InputContext {
    pub(super) node: NodeId,
    pub(super) surrounding: Option<(String, i32, i32)>,
    pub(super) rect: Option<(i32, i32, i32, i32)>,
}
