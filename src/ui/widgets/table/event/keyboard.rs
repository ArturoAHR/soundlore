use std::hash::Hash;

use iced::{
    advanced::{Shell, renderer},
    keyboard::{self},
};

use crate::{
    traits::Identifiable,
    ui::widgets::table::{Catalog, Table, TableRow, state::State},
};

impl<'a, T, ColumnId, Message, Theme, Renderer> Table<'a, '_, T, ColumnId, Message, Theme, Renderer>
where
    T: Identifiable + TableRow,
    T::Identifier: Hash + Eq + Clone,
    ColumnId: Hash + Eq + Clone,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn handle_keyboard_event(
        &self,
        state: &mut State<T::Identifier, ColumnId>,
        shell: &mut Shell<'_, Message>,
        event: &keyboard::Event,
    ) {
        match event {
            keyboard::Event::KeyPressed { key, modifiers, .. }
                if matches!(key.as_ref(), keyboard::Key::Character("a"))
                    && *modifiers == keyboard::Modifiers::COMMAND =>
            {
                self.handle_keyboard_select_all_command(state, shell);
            }

            keyboard::Event::ModifiersChanged(modifiers) => {
                state.keyboard_modifiers = *modifiers;
            }

            _ => {}
        }
    }
}
