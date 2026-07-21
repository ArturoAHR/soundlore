use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use iced::{
    Element,
    advanced::{renderer, widget::Tree},
    keyboard,
};

use crate::ui::widgets::table::event::mouse::{MouseInteraction, TableArea, TableClick};

pub struct State<RowId, ColumnId>
where
    RowId: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
{
    /// Table cells state, handled on our side since virtualization plays poorly with
    /// normal children state management.
    pub cell_state: CellState<RowId, ColumnId>,

    /// Table vertical scroll offset.
    pub offset_y: f32,

    /// The id of the row used as an anchor for multi-selection operations.
    pub selection_anchor_row_id: Option<RowId>,

    /// Last mouse click (used to determine double clicks).
    pub previous_click: Option<TableClick<RowId, ColumnId>>,

    /// Tracks the start point of a dragging action.
    pub mouse_interaction: MouseInteraction<RowId, ColumnId>,

    /// Currently pressed keyboard modifiers.
    pub keyboard_modifiers: keyboard::Modifiers,

    /// Table and Window focus status.
    pub focus_state: FocusState,

    pub last_layout_invalidation_state: LayoutState,

    pub last_redraw_request_state: DrawState<RowId, ColumnId>,
}

impl<RowId, ColumnId> State<RowId, ColumnId>
where
    RowId: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
{
    /// Determines if we should request a redraw, it will automatically update the necessary
    /// internal state on its own.
    pub fn take_pending_redraw_request(&mut self) -> bool {
        if self.last_redraw_request_state.mouse_interaction_area != self.mouse_interaction.area {
            self.last_redraw_request_state.mouse_interaction_area =
                self.mouse_interaction.area.clone();

            return true;
        }

        false
    }

    /// Determines if we should invalidate the current layout, it will automatically update the necessary
    /// internal state on its own.
    pub fn take_pending_layout_invalidation(&mut self) -> bool {
        if (self.last_layout_invalidation_state.offset_y - self.offset_y).abs() > 0.1 {
            self.last_layout_invalidation_state.offset_y = self.offset_y;

            return true;
        }

        false
    }
}

impl<RowId, ColumnId> Default for State<RowId, ColumnId>
where
    RowId: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
{
    fn default() -> Self {
        Self {
            cell_state: CellState::default(),
            focus_state: FocusState::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            last_layout_invalidation_state: LayoutState::default(),
            last_redraw_request_state: DrawState::default(),
            mouse_interaction: MouseInteraction::default(),
            offset_y: 0.0,
            previous_click: None,
            selection_anchor_row_id: None,
        }
    }
}

#[derive(Default)]
pub struct LayoutState {
    offset_y: f32,
}

pub struct DrawState<RowId, ColumnId> {
    mouse_interaction_area: Option<TableArea<RowId, ColumnId>>,
}

impl<RowId, ColumnId> Default for DrawState<RowId, ColumnId> {
    fn default() -> Self {
        Self {
            mouse_interaction_area: None,
        }
    }
}

#[derive(Debug)]
pub struct FocusState {
    pub widget: bool,
    pub window: bool,
}

impl FocusState {
    pub fn is_focused(&self) -> bool {
        self.widget && self.window
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            widget: false,
            window: true,
        }
    }
}

pub struct CellState<RowId, ColumnId>
where
    RowId: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
{
    rows: HashMap<RowId, HashMap<ColumnId, Tree>>,
}

impl<RowId, ColumnId> CellState<RowId, ColumnId>
where
    RowId: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
{
    /// Gets a reference to the cell state
    pub fn get(&self, row_id: &RowId, column_id: &ColumnId) -> Option<&Tree> {
        self.rows.get(row_id)?.get(column_id)
    }

    /// Gets a mutable reference to cell state
    pub fn get_mut(&mut self, row_id: &RowId, column_id: &ColumnId) -> Option<&mut Tree> {
        self.rows.get_mut(row_id)?.get_mut(column_id)
    }

    /// Gets the cell state or inserts a newly created one and returns it
    pub fn get_mut_or_insert<Message, Theme, Renderer>(
        &mut self,
        row_id: &RowId,
        column_id: &ColumnId,
        cell: &Element<'_, Message, Theme, Renderer>,
    ) -> &mut Tree
    where
        Renderer: renderer::Renderer,
    {
        if self.get(row_id, column_id).is_some() {
            return self.get_mut(row_id, column_id).map_or_else(
                || unreachable!(),
                |tree| {
                    tree.diff(cell.as_widget());

                    tree
                },
            );
        }

        self.insert(row_id, column_id, Tree::new(cell.as_widget()));

        self.get_mut(row_id, column_id)
            .unwrap_or_else(|| unreachable!())
    }

    /// Inserts the cell state for a row id and column id
    pub fn insert(&mut self, row_id: &RowId, column_id: &ColumnId, state: Tree) {
        let row_cell_states = self.rows.entry(row_id.clone()).or_default();

        row_cell_states.insert(column_id.clone(), state);
    }

    /// Removes row ids that are not within the provided set
    pub fn prune(&mut self, row_ids: &HashSet<&RowId>) {
        self.rows.retain(|row_id, _| row_ids.contains(row_id));
    }
}

impl<RowId, ColumnId> Default for CellState<RowId, ColumnId>
where
    RowId: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
{
    fn default() -> Self {
        Self {
            rows: HashMap::new(),
        }
    }
}
