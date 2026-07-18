use iced::{
    advanced::{Shell, renderer},
    keyboard::{self},
};

use crate::ui::widgets::table::{
    Catalog, Table,
    state::{Identifiable, State},
};

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub(in super::super) fn handle_keyboard_event(
        &self,
        state: &mut State,
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
