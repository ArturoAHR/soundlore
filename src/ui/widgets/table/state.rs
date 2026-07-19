use std::collections::{HashMap, HashSet};

use iced::{
    Element,
    advanced::{renderer, widget::Tree},
    keyboard,
};

use crate::ui::widgets::table::mouse::{MouseInteraction, TableArea, TableClick};

pub const HEADERS_ROW_IDENTIFIER: &str = "headers-row";

pub type TableIdentifier = String;

pub trait Identifiable {
    fn id(&self) -> &TableIdentifier;
}

#[derive(Default)]
pub struct State {
    /// Table cells state, handled on our side since virtualization plays poorly with
    /// normal children state management.
    pub cell_state: CellState,

    /// Table vertical scroll offset.
    pub offset_y: f32,

    /// The id of the row used as an anchor for multi-selection operations.
    pub selection_anchor_row_id: Option<TableIdentifier>,

    /// Last mouse click (used to determine double clicks).
    pub previous_click: Option<TableClick>,

    /// Tracks the start point of a dragging action.
    pub mouse_interaction: MouseInteraction,

    /// Currently pressed keyboard modifiers.
    pub keyboard_modifiers: keyboard::Modifiers,

    /// Table and Window focus status.
    pub focus_state: FocusState,

    pub last_layout_invalidation_state: LayoutState,

    pub last_redraw_request_state: DrawState,
}

impl State {
    /// Determines if we should request a redraw, it will automatically update the necessary
    /// internal state on its own.
    pub fn is_pending_redraw_request(&mut self) -> bool {
        if self.last_redraw_request_state.mouse_interaction_area != self.mouse_interaction.area {
            self.last_redraw_request_state.mouse_interaction_area =
                self.mouse_interaction.area.clone();

            return true;
        }

        false
    }

    /// Determines if we should invalidate the current layout, it will automatically update the necessary
    /// internal state on its own.
    pub fn is_pending_layout_invalidation(&mut self) -> bool {
        if (self.last_layout_invalidation_state.offset_y - self.offset_y).abs() > 0.1 {
            self.last_layout_invalidation_state.offset_y = self.offset_y;

            return true;
        }

        false
    }
}

#[derive(Default)]
pub struct LayoutState {
    offset_y: f32,
}

#[derive(Default)]
pub struct DrawState {
    mouse_interaction_area: Option<TableArea>,
}

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

#[derive(Default)]
pub struct CellState {
    rows: HashMap<String, HashMap<String, Tree>>,
}

impl CellState {
    /// Gets a reference to the cell state
    pub fn get(&self, row_id: &str, column_id: &str) -> Option<&Tree> {
        self.rows.get(row_id)?.get(column_id)
    }

    /// Gets a mutable reference to cell state
    pub fn get_mut(&mut self, row_id: &str, column_id: &str) -> Option<&mut Tree> {
        self.rows.get_mut(row_id)?.get_mut(column_id)
    }

    /// Gets the cell state or inserts a newly created one and returns it
    pub fn get_mut_or_insert<Message, Theme, Renderer>(
        &mut self,
        row_id: &str,
        column_id: &str,
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
    pub fn insert(&mut self, row_id: &str, column_id: &str, state: Tree) {
        let row_cell_states = self.rows.entry(row_id.to_owned()).or_default();

        row_cell_states.insert(column_id.to_owned(), state);
    }

    /// Removes row ids that are not within the provided set
    pub fn prune(&mut self, row_ids: &HashSet<&String>) {
        self.rows.retain(|row_id, _| row_ids.contains(row_id));
    }
}
