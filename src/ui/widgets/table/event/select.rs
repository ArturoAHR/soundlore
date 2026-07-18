use iced::{
    Point, Rectangle,
    advanced::{Shell, mouse::click, renderer},
};

use crate::ui::{
    utils::select::{SelectOperation, select_values},
    widgets::table::{
        Catalog, Table,
        state::{Identifiable, State},
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
        bounds: Rectangle,
        cursor_position: Point,
    ) {
        let Some(clicked_column_id) =
            self.get_header_column_id_at_position(bounds, cursor_position)
        else {
            return;
        };

        if let Some(on_header_cell_click) = self.on_header_cell_click.as_ref() {
            shell.publish(on_header_cell_click(clicked_column_id));
            shell.capture_event();
        }
    }

    pub fn handle_mouse_row_click(
        &self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        bounds: Rectangle,
        cursor_position: Point,
        click_kind: click::Kind,
    ) {
        let Some(clicked_row_id) = self.get_row_id_at_position(bounds, cursor_position) else {
            return;
        };

        if let Some(on_row_select) = self.on_row_select.as_ref() {
            shell.capture_event();

            let row_ids = self.records.iter().map(Identifiable::id);

            let (selected_row_ids, anchor_row_id) = select_values(
                row_ids,
                self.selected_rows.iter(),
                SelectOperation::from_keyboard_modifiers(
                    state.keyboard_modifiers,
                    clicked_row_id,
                    state.selection_anchor_row_id.as_ref(),
                ),
            );

            state.selection_anchor_row_id = anchor_row_id;

            shell.publish(on_row_select(selected_row_ids));
        }

        if let Some(on_row_double_click) = self.on_row_double_click.as_ref()
            && matches!(click_kind, click::Kind::Double)
        {
            shell.publish(on_row_double_click(clicked_row_id.to_owned()));
            shell.capture_event();
        }
    }

    pub fn handle_mouse_row_drag(
        &self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        bounds: Rectangle,
        cursor_position: Point,
    ) {
        if let Some(on_row_select) = self.on_row_select.as_ref() {
            let Some(current_position_row_id) =
                self.get_row_id_at_position(bounds, cursor_position)
            else {
                return;
            };

            let row_ids = self.records.iter().map(Identifiable::id);

            let mut select_operation = SelectOperation::from_keyboard_modifiers(
                state.keyboard_modifiers,
                current_position_row_id,
                state.selection_anchor_row_id.as_ref(),
            );

            if matches!(
                select_operation,
                SelectOperation::Single { .. } | SelectOperation::Toggle { .. }
            ) {
                select_operation = SelectOperation::Range {
                    target_value: current_position_row_id,
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
