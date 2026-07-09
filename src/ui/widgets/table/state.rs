use std::collections::{HashMap, HashSet};

use iced::{
    Element,
    advanced::{renderer, widget::Tree},
};

pub const HEADERS_ROW_IDENTIFIER: &str = "headers-row";

pub type TableIdentifier = String;

pub trait Identifiable {
    fn id(&self) -> &TableIdentifier;
}

#[derive(Default)]
pub struct State {
    pub offset_y: f32,
    pub cell_state: CellState,
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
    pub fn get_mut_or_insert<'a, Message, Theme, Renderer>(
        &mut self,
        row_id: &str,
        column_id: &str,
        cell: &Element<'a, Message, Theme, Renderer>,
    ) -> &mut Tree
    where
        Renderer: renderer::Renderer,
    {
        if self.get(row_id, column_id).is_some() {
            return match self.get_mut(row_id, column_id) {
                Some(tree) => {
                    tree.diff(cell.as_widget());

                    tree
                }
                None => unreachable!(),
            };
        }

        self.insert(row_id, column_id, Tree::new(cell.as_widget()));

        match self.get_mut(row_id, column_id) {
            Some(tree) => tree,
            None => unreachable!(),
        }
    }

    /// Inserts the cell state for a row id and column id
    pub fn insert(&mut self, row_id: &str, column_id: &str, state: Tree) {
        let row_cell_states = self.rows.entry(row_id.to_owned()).or_insert(HashMap::new());

        row_cell_states.insert(column_id.to_owned(), state);
    }

    /// Removes row ids that are not within the provided set
    pub fn prune(&mut self, row_ids: HashSet<&String>) {
        self.rows.retain(|row_id, _| row_ids.contains(row_id));
    }
}
