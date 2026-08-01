#![doc = include_str!("command_dispatch.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::{EventDispatcher, UiEvent};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{EVENT_CHANGE, EVENT_CLICK};

use super::{AncestorChain, CommandTarget, HandlerRegistry};

#[doc = include_str!("command_dispatch.doc/CommandDispatchOutcome.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandDispatchOutcome {
    #[doc = include_str!("command_dispatch.doc/claimed.md")]
    pub(crate) claimed: bool,
    #[doc = include_str!("command_dispatch.doc/anything_ran.md")]
    pub(crate) anything_ran: bool,
}

#[doc = include_str!("command_dispatch.doc/CommandDispatch.md")]
pub(crate) struct CommandDispatch<'a> {
    #[doc = include_str!("command_dispatch.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("command_dispatch.doc/routing.md")]
    routing: &'a mut HandlerRegistry,
}

impl<'a> CommandDispatch<'a> {
    #[doc = include_str!("command_dispatch.doc/new.md")]
    pub(crate) fn new(tree: &'a RetainedTree, routing: &'a mut HandlerRegistry) -> Self {
        Self { tree, routing }
    }

    #[doc = include_str!("command_dispatch.doc/binding_for.md")]
    pub(crate) fn binding_for(
        tree: &RetainedTree,
        target: NodeId,
        event: InternedSymbol,
    ) -> Option<(NodeId, InternedSymbol)> {
        AncestorChain::of(tree, Some(target))
            .into_iter()
            .find_map(|node| {
                tree.get(node)
                    .and_then(|n| n.command_for_event(event))
                    .map(|command| (node, command))
            })
    }

    #[doc = include_str!("command_dispatch.doc/activation_command_for.md")]
    pub(crate) fn activation_command_for(
        tree: &RetainedTree,
        target: NodeId,
        editable: bool,
    ) -> Option<(NodeId, InternedSymbol)> {
        if let Some(click) = Self::binding_for(tree, target, EVENT_CLICK) {
            return Some(click);
        }
        // `EVENT_CHANGE`-on-click is a toggle/select activation only; for an
        // editable field `change` is a commit event, so never resolve it here.
        if editable {
            return None;
        }
        Self::binding_for(tree, target, EVENT_CHANGE)
    }

    #[doc = include_str!("command_dispatch.doc/dispatch.md")]
    pub(crate) fn dispatch(
        &mut self,
        origin: Option<NodeId>,
        command: InternedSymbol,
        envelope: UiEvent,
    ) -> CommandDispatchOutcome {
        if !self.routing.permits_dispatch(command) {
            return CommandDispatchOutcome {
                claimed: false,
                anything_ran: false,
            };
        }
        let mut event = match origin {
            Some(node) => envelope.at_target(node),
            None => envelope,
        };
        // §2.3 steps 1-3: capture, target, bubble.
        let listeners_run = self.propagate(origin, command, &mut event);
        // §2.3 step 4: the default action — §2.6's routing chain.
        let claimed = if event.is_default_prevented() {
            false
        } else {
            let route = Self::route(self.tree, self.routing, origin);
            self.routing.dispatch_along(&route, command, &mut event)
        };
        CommandDispatchOutcome {
            claimed,
            anything_ran: claimed || listeners_run > 0,
        }
    }

    #[doc = include_str!("command_dispatch.doc/route.md")]
    pub(crate) fn route(
        tree: &RetainedTree,
        routing: &HandlerRegistry,
        origin: Option<NodeId>,
    ) -> Vec<CommandTarget> {
        let mut route = Vec::new();
        if let Some(origin) = origin {
            // 1. the focused node, and 3. its ancestors — `AncestorChain`
            //    yields the node itself first, then each logical parent.
            let chain = AncestorChain::of(tree, Some(origin));
            route.push(CommandTarget::Node(origin));
            // 2. every open scope containing the origin, innermost (last opened)
            //    first — `scopes` is a stack, so reverse iteration is inside-out.
            route.extend(
                routing
                    .scopes()
                    .iter()
                    .rev()
                    .filter(|scope| scope.contains(origin))
                    .map(|scope| CommandTarget::Scope(scope.id())),
            );
            route.extend(chain.into_iter().skip(1).map(CommandTarget::Node));
        }
        // 4. this window surface, then 5. the application fallback.
        route.push(CommandTarget::Window);
        route.push(CommandTarget::App);
        route
    }

    #[doc = include_str!("command_dispatch.doc/propagate.md")]
    fn propagate(
        &mut self,
        origin: Option<NodeId>,
        command: InternedSymbol,
        event: &mut UiEvent,
    ) -> usize {
        // With no listener registered anywhere there is nothing to run, and
        // computing a path to hand to nobody would cost two allocations per
        // click. This is also what makes the phases free for an application that
        // never asks for them: the walk is skipped, not merely empty.
        let Some(target) = origin.filter(|_| self.routing.has_listeners()) else {
            return 0;
        };
        // `AncestorChain` yields the node first and the root last; the dispatcher
        // wants root → target, which is the reverse.
        let mut root_to_target = AncestorChain::of(self.tree, Some(target)).into_vec();
        root_to_target.reverse();

        let mut run = 0;
        for step in EventDispatcher::new().path(&root_to_target) {
            if event.is_stopped() {
                break;
            }
            event.phase = step.phase;
            event.current_target = Some(step.node);
            run += self
                .routing
                .run_listeners(step.node, command, step.phase, event);
        }
        run
    }
}

#[cfg(test)]
mod tests {
    use obscur_interaction::event::EventOrigin;

    use super::*;

    #[test]
    fn a_route_with_no_origin_is_the_two_non_positional_links() {
        let tree = RetainedTree::new();
        let routing = HandlerRegistry::new();
        assert_eq!(
            CommandDispatch::route(&tree, &routing, None),
            vec![CommandTarget::Window, CommandTarget::App]
        );
    }

    #[test]
    fn a_route_is_never_empty_even_for_a_node_that_is_not_in_the_tree() {
        let tree = RetainedTree::new();
        let routing = HandlerRegistry::new();
        let stale = NodeId::from_raw(3);
        assert_eq!(
            CommandDispatch::route(&tree, &routing, Some(stale)),
            vec![
                CommandTarget::Node(stale),
                CommandTarget::Window,
                CommandTarget::App
            ]
        );
    }

    #[test]
    fn a_dispatch_that_nothing_claims_owes_no_frame() {
        let tree = RetainedTree::new();
        let mut routing = HandlerRegistry::new();
        let mut dispatch = CommandDispatch::new(&tree, &mut routing);
        let outcome = dispatch.dispatch(
            None,
            InternedSymbol::named("save"),
            UiEvent::detached(EventOrigin::Synthetic),
        );
        assert_eq!(
            outcome,
            CommandDispatchOutcome {
                claimed: false,
                anything_ran: false,
            }
        );
    }
}
