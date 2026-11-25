//! Hierarchical table rows for application-owned collections.

use std::{num::NonZeroU32, sync::Arc};

use obscur_identity::InternedSymbol;
use obscur_interaction::command::NamedKey;
use obscur_view::view::{STYLE, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use super::{Tree, TreeNode};
use crate::table::{Column, Table};

/// Combines tree visibility and selection with keyed table rows and cells.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeGrid {
    tree: Tree,
    table: Table,
}

impl TreeGrid {
    /// Creates an empty tree grid.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a column header. The first column holds tree labels.
    #[must_use]
    pub fn column(mut self, column: Column) -> Self {
        self.table = self.table.column(column);
        self
    }

    /// Applies an inline CSS sticky header with a top inset of zero. It follows the runtime viewport’s sticky-positioning path.
    #[must_use]
    pub fn sticky_header(mut self, sticky: bool) -> Self {
        self.table = self.table.sticky_header(sticky);
        self
    }

    /// Moves a column without changing the hierarchy or cell identity.
    pub fn move_column(&mut self, from: usize, to: usize) -> bool {
        self.table.move_column(from, to)
    }

    /// Sets a column width in logical pixels. None restores automatic sizing.
    pub fn resize_column(&mut self, column: usize, width: Option<u32>) -> bool {
        self.table.resize_column(column, width)
    }

    /// Appends a node in tree preorder. Additional cells follow its label.
    #[must_use]
    pub fn row<I, S>(mut self, node: TreeNode, cells: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Box<str>>,
    {
        let label: Box<str> = node.label().into();
        self.table = self
            .table
            .row(std::iter::once(label).chain(cells.into_iter().map(Into::into)));
        self.tree = self.tree.node(node);
        self
    }

    /// Expands or collapses the branch at a logical row index.
    pub fn set_expanded(&mut self, index: usize, expanded: bool) -> bool {
        if expanded {
            self.tree.expand(index)
        } else if self.tree.collapse(index) {
            if let Some(selected) = self.tree.selected_index()
                && !self.tree.visible_nodes().contains(&selected)
            {
                self.tree.select(index);
            }
            true
        } else {
            false
        }
    }

    /// Selects the row at its logical index.
    pub fn select(&mut self, index: usize) -> bool {
        self.tree.visible_nodes().contains(&index) && self.tree.select(index)
    }

    /// Moves selection by a signed number of visible rows. Collapsed descendants are skipped.
    pub fn move_selection(&mut self, delta: isize) -> bool {
        if delta == 0 {
            return false;
        }
        let visible = self.tree.visible_nodes();
        let Some(last) = visible.len().checked_sub(1) else {
            return false;
        };
        let selected = self.tree.selected_index();
        let target = selected
            .and_then(|index| visible.iter().position(|&row| row == index))
            .map_or(if delta < 0 { last } else { 0 }, |index| {
                index.saturating_add_signed(delta).min(last)
            });
        let target = visible[target];
        selected != Some(target) && self.tree.select(target)
    }

    /// Applies arrow, Home, and End navigation. Left and right traverse or toggle branches. Returns whether selection or expansion changed.
    pub fn navigate(&mut self, key: NamedKey) -> bool {
        match key {
            NamedKey::ArrowUp => self.move_selection(-1),
            NamedKey::ArrowDown => self.move_selection(1),
            NamedKey::Home => self.select_edge(false),
            NamedKey::End => self.select_edge(true),
            NamedKey::ArrowLeft | NamedKey::ArrowRight => self.navigate_branch(key),
            _ => false,
        }
    }

    fn select_edge(&mut self, last: bool) -> bool {
        let visible = self.tree.visible_nodes();
        let edge = if last {
            visible.last()
        } else {
            visible.first()
        };
        edge.is_some_and(|&row| self.selected_index() != Some(row) && self.tree.select(row))
    }

    fn navigate_branch(&mut self, key: NamedKey) -> bool {
        let Some(index) = self.selected_index() else {
            return false;
        };
        let Some(node) = self.tree.node_at(index) else {
            return false;
        };
        let depth = node.depth();
        if key == NamedKey::ArrowRight {
            if !node.has_children() {
                return false;
            }
            if !self.tree.is_expanded(index) {
                return self.set_expanded(index, true);
            }
            return self
                .tree
                .node_at(index + 1)
                .is_some_and(|child| child.depth() > depth)
                && self.select(index + 1);
        }
        if self.tree.is_expanded(index) {
            return self.set_expanded(index, false);
        }
        let parent = (0..index).rev().find(|&row| {
            self.tree
                .node_at(row)
                .is_some_and(|parent| parent.depth() < depth)
        });
        parent.is_some_and(|row| self.select(row))
    }

    /// Returns the selected logical row index.
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.tree.selected_index()
    }

    /// Renders currently visible rows and exposes selection and expansion through accessibility.
    #[must_use]
    pub fn view(&self) -> View {
        self.table
            .view_rows("treegrid", self.tree.visible_nodes(), |index, row| {
                self.configure_row(index, row, None)
            })
    }

    /// Maps a virtual list slot to its logical row. Slot zero is the header.
    #[must_use]
    pub fn logical_row_at(&self, virtual_slot: usize) -> Option<usize> {
        self.tree
            .visible_nodes()
            .get(virtual_slot.checked_sub(1)?)
            .copied()
    }

    /// Maps a visible logical row to its virtual list slot. Slot zero is reserved for the header.
    #[must_use]
    pub fn virtual_slot_of(&self, logical_row: usize) -> Option<usize> {
        self.tree
            .visible_nodes()
            .iter()
            .position(|&row| row == logical_row)
            .map(|slot| slot + 1)
    }

    /// Renders the tree grid through the runtime’s fixed-height virtual list. Give each list a stable unique key and install the inline CSS compiler. The header is the first item and scrolls with the rows. The tree grid element can be styled as the scroll container.
    #[must_use]
    pub fn virtual_view(self: Arc<Self>, list: InternedSymbol, row_height: NonZeroU32) -> View {
        let visible = self.tree.visible_nodes();
        let grid = Arc::clone(&self);
        let height = row_height.get();
        let rows = ViewNode::virtual_list_with_rows(
            list,
            visible.len() + 1,
            height as f32,
            move |index| {
                let row = if index == 0 {
                    grid.table.header_row(Some(height))
                } else {
                    let index = visible[index - 1];
                    grid.configure_row(index, grid.table.data_row(index), Some(height))
                };
                ViewFragment::from_nodes(vec![row])
            },
        );
        self.table.view_with_children("treegrid", vec![rows])
    }

    /// Applies tree state and optional fixed row sizing to a rendered table row.
    fn configure_row(&self, index: usize, mut row: ViewNode, height: Option<u32>) -> ViewNode {
        let node = self.tree.node_at(index).expect("visible tree row");
        row = row.with_prop(ViewProperty::new(
            InternedSymbol::named("aria-selected"),
            ViewValue::Bool(self.tree.selected_index() == Some(index)),
        ));
        if node.has_children() {
            row = row.with_prop(ViewProperty::new(
                InternedSymbol::named("aria-expanded"),
                ViewValue::Bool(self.tree.is_expanded(index)),
            ));
        }
        let mut style = format!("--obscur-tree-depth:{}", node.depth());
        if let Some(height) = height {
            use std::fmt::Write;
            let _ = write!(style, ";box-sizing:border-box;height:{height}px");
        }
        row.with_prop(ViewProperty::new(STYLE, ViewValue::from(style)))
    }
}
