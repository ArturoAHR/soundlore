use std::{collections::HashSet, hash::BuildHasher, iter};

use iced::keyboard;

use crate::ui::widgets::table::state::TableIdentifier;

enum SelectKeyboardModifier {
    Default,
    Control,
    Shift,
    ControlShift,
}

impl SelectKeyboardModifier {
    pub fn from_keyboard_modifiers(keyboard_modifiers: keyboard::Modifiers) -> Self {
        if keyboard_modifiers.command() && keyboard_modifiers.shift() {
            return Self::ControlShift;
        }

        if keyboard_modifiers.command() {
            return Self::Control;
        }

        if keyboard_modifiers.shift() {
            return Self::Shift;
        }

        Self::Default
    }
}

pub fn get_new_selected_row_ids<S: BuildHasher>(
    row_ids: &[TableIdentifier],
    selected_row_ids: &HashSet<&TableIdentifier, S>,
    target_row_id: &TableIdentifier,
    anchor_row_id: &TableIdentifier,
    keyboard_modifiers: keyboard::Modifiers,
) -> (HashSet<TableIdentifier>, TableIdentifier) {
    if row_ids.is_empty() {
        return get_default_return(selected_row_ids, anchor_row_id);
    }

    let Some(target_row_index) = row_ids.iter().position(|row_id| row_id == target_row_id) else {
        return get_default_return(selected_row_ids, anchor_row_id);
    };

    let mut anchor_row_id = anchor_row_id;
    let anchor_row_index = row_ids
        .iter()
        .position(|row_id| row_id == anchor_row_id)
        .unwrap_or(0);

    if let Some(first_row_id) = row_ids.first()
        && anchor_row_index == 0
        && first_row_id != anchor_row_id
    {
        anchor_row_id = first_row_id;
    }

    let select_keyboard_modifiers =
        SelectKeyboardModifier::from_keyboard_modifiers(keyboard_modifiers);

    match select_keyboard_modifiers {
        SelectKeyboardModifier::Default => (
            HashSet::from_iter([target_row_id.to_owned()]),
            target_row_id.to_owned(),
        ),
        SelectKeyboardModifier::Shift => {
            if target_row_index <= anchor_row_index {
                (
                    row_ids[target_row_index..=anchor_row_index]
                        .iter()
                        .cloned()
                        .collect(),
                    anchor_row_id.to_owned(),
                )
            } else {
                (
                    row_ids[anchor_row_index..=target_row_index]
                        .iter()
                        .cloned()
                        .collect(),
                    anchor_row_id.to_owned(),
                )
            }
        }
        SelectKeyboardModifier::Control => {
            if selected_row_ids.contains(target_row_id) {
                (
                    selected_row_ids
                        .iter()
                        .copied()
                        .filter(|&row_id| row_id != target_row_id)
                        .cloned()
                        .collect(),
                    target_row_id.to_owned(),
                )
            } else {
                (
                    iter::once(target_row_id.to_owned())
                        .chain(selected_row_ids.iter().copied().cloned())
                        .collect(),
                    target_row_id.to_owned(),
                )
            }
        }
        SelectKeyboardModifier::ControlShift => {
            let mut new_selected_row_ids: HashSet<TableIdentifier> =
                selected_row_ids.iter().copied().cloned().collect();

            let selected_row_id_range = if target_row_index <= anchor_row_index {
                target_row_index..=anchor_row_index
            } else {
                anchor_row_index..=target_row_index
            };

            new_selected_row_ids.extend(row_ids[selected_row_id_range].iter().cloned());

            (new_selected_row_ids, anchor_row_id.to_owned())
        }
    }
}

fn get_default_return<S: BuildHasher>(
    selected_row_ids: &HashSet<&TableIdentifier, S>,
    anchor_row_id: &TableIdentifier,
) -> (HashSet<TableIdentifier>, TableIdentifier) {
    (
        selected_row_ids.iter().copied().cloned().collect(),
        anchor_row_id.to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn get_iterator(input: &str) -> impl Iterator<Item = TableIdentifier> {
        input.chars().map(Into::<String>::into)
    }

    fn get_row_ids() -> Vec<TableIdentifier> {
        get_iterator("abcdefghijklmnopqrstuvwxyz").collect()
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assert_selected_row_ids(selected_row_ids: HashSet<String>, expected_selected_row_ids: &str) {
        assert_eq!(
            selected_row_ids.len(),
            expected_selected_row_ids.len(),
            "Amount of selected row ids does not match the expected amount

        selected_row_ids: {selected_row_ids:?},
        expected_selected_row_ids (string): \"{expected_selected_row_ids}\"
            ",
        );

        let expected_row_ids = get_iterator(expected_selected_row_ids);

        for row_id in expected_row_ids {
            assert!(
                selected_row_ids.contains(&row_id),
                "Selected row id set does not contain the following expected row id: {row_id}."
            );
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assert_anchor_row_id(anchor_row_id: TableIdentifier, expected_anchor_row_id: &str) {
        assert_eq!(anchor_row_id, expected_anchor_row_id.to_owned());
    }

    #[test]
    fn should_get_selected_rows_without_modifiers() {
        let row_ids = get_row_ids();

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let keyboard_modifiers = keyboard::Modifiers::empty();

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_without_modifiers_for_an_already_selected_row() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("a").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let keyboard_modifiers = keyboard::Modifiers::empty();

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_without_modifiers_and_not_include_previously_selected_rows() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("cdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let keyboard_modifiers = keyboard::Modifiers::empty();

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_select() {
        let row_ids = get_row_ids();

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_unselect() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("a").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "a".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_select_and_maintain_existing_selected_rows()
     {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("cdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "acdefgh");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_unselect_and_maintain_existing_selected_rows()
     {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("acdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "cdefgh");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select() {
        let row_ids = get_row_ids();

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select_with_already_selected_rows() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("cdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select_already_selected_row() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("c").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "c".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "c");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_when_anchor_row_id_is_not_in_row_ids() {
        let row_ids = get_row_ids();

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "c".to_owned();
        let anchor_row_id = "!".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier() {
        let row_ids = get_row_ids();

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_with_already_selected_rows() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("cdef").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abcdef");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_already_selected_rows() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("abc").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_a_set_of_non_contiguous_rows()
     {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("wxyz").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abcwxyz");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_the_whole_table() {
        let row_ids = get_row_ids();

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "z".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, &row_ids.join(""));
        assert_anchor_row_id(new_anchor_row_id, "z");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_the_whole_table_with_existing_selections()
     {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("gdrfxz").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "z".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, &row_ids.join(""));
        assert_anchor_row_id(new_anchor_row_id, "z");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_when_anchor_row_id_is_not_in_row_ids()
     {
        let row_ids = get_row_ids();

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "c".to_owned();
        let anchor_row_id = "!".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_return_selected_rows_given_when_target_row_id_is_not_found_in_row_ids() {
        let row_ids = get_row_ids();

        let selected_row_ids: Vec<String> = get_iterator("gdrfxz").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "!".to_owned();
        let anchor_row_id = "z".to_owned();

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "gdrfxz");
        assert_anchor_row_id(new_anchor_row_id, "z");
    }

    #[test]
    fn should_return_selected_rows_given_when_row_ids_is_empty() {
        let row_ids = Vec::new();

        let selected_row_ids: Vec<String> = get_iterator("def").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "z".to_owned();

        let keyboard_modifiers = keyboard::Modifiers::empty();

        let (new_selected_row_ids, new_anchor_row_id) = get_new_selected_row_ids(
            &row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            keyboard_modifiers,
        );

        assert_selected_row_ids(new_selected_row_ids, "def");
        assert_anchor_row_id(new_anchor_row_id, "z");
    }
}
