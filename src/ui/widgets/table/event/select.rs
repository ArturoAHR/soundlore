use iced::advanced::{Shell, mouse::click, renderer};

use crate::ui::{
    utils::select::{SelectOperation, select_values},
    widgets::table::{
        Catalog, Table,
        mouse::TableArea,
        state::{Identifiable, State, TableIdentifier},
    },
};

impl<'a, T, Message, Theme, Renderer> Table<'a, T, Message, Theme, Renderer>
where
    T: Identifiable,
    Message: 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn handle_mouse_header_click(
        &self,
        shell: &mut Shell<'_, Message>,
        column_id: TableIdentifier,
    ) {
        if let Some(on_header_cell_click) = self.on_header_cell_click.as_ref() {
            shell.publish(on_header_cell_click(column_id));
            shell.capture_event();
        }
    }

    pub fn handle_mouse_row_click(
        &self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        row_id: TableIdentifier,
        click_kind: click::Kind,
    ) {
        if let Some(on_row_select) = self.on_row_select.as_ref() {
            shell.capture_event();

            let row_ids = self.records.iter().map(Identifiable::id);

            let (selected_row_ids, anchor_row_id) = select_values(
                row_ids,
                self.selected_rows.iter(),
                SelectOperation::from_keyboard_modifiers(
                    state.keyboard_modifiers,
                    &row_id,
                    state.selection_anchor_row_id.as_ref(),
                ),
            );

            state.selection_anchor_row_id = anchor_row_id;

            shell.publish(on_row_select(selected_row_ids));
        }

        if let Some(on_row_double_click) = self.on_row_double_click.as_ref()
            && matches!(click_kind, click::Kind::Double)
        {
            shell.publish(on_row_double_click(row_id));
            shell.capture_event();
        }
    }

    pub fn handle_mouse_row_drag(&self, state: &mut State, shell: &mut Shell<'_, Message>) {
        // TODO (v2): Add scroll on moving the mouse up or down the table body past a certain threshold
        if let Some(on_row_select) = self.on_row_select.as_ref() {
            let Some(TableArea::Body {
                row_id: Some(row_id),
            }) = state.mouse_interaction.area.as_ref()
            else {
                return;
            };

            let row_ids = self.records.iter().map(Identifiable::id);

            let mut select_operation = SelectOperation::from_keyboard_modifiers(
                state.keyboard_modifiers,
                row_id,
                state.selection_anchor_row_id.as_ref(),
            );

            if matches!(
                select_operation,
                SelectOperation::Single { .. } | SelectOperation::Toggle { .. }
            ) {
                select_operation = SelectOperation::Range {
                    target_value: row_id,
                    anchor_value: state.selection_anchor_row_id.as_ref(),
                }
            }

            let (selected_row_ids, anchor_row_id) =
                select_values(row_ids, self.selected_rows.iter(), select_operation);

            state.selection_anchor_row_id = anchor_row_id;

            shell.publish(on_row_select(selected_row_ids));
        }
    }

    pub fn handle_keyboard_select_all_command(
        &self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(on_row_select) = self.on_row_select.as_ref()
            && state.focus_state.is_focused()
        {
            shell.capture_event();

            let row_ids = self.records.iter().map(Identifiable::id);

            let (selected_row_ids, anchor_row_id) = select_values(
                row_ids,
                self.selected_rows.iter(),
                SelectOperation::All {
                    anchor_value: state.selection_anchor_row_id.as_ref(),
                },
            );

            state.selection_anchor_row_id = anchor_row_id;

            shell.publish(on_row_select(selected_row_ids));
        }
    }
}
