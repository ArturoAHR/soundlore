use std::{
    cmp::{max, min},
    collections::HashSet,
    hash::BuildHasher,
    iter,
};

use iced::keyboard;

use crate::ui::widgets::table::state::TableIdentifier;

#[derive(Debug, Clone, Copy)]
pub enum SelectOperation {
    Single,
    Toggle,
    Range,
    Union,
}

impl SelectOperation {
    pub fn from_keyboard_modifiers(keyboard_modifiers: keyboard::Modifiers) -> Self {
        if keyboard_modifiers.command() && keyboard_modifiers.shift() {
            return Self::Union;
        }

        if keyboard_modifiers.command() {
            return Self::Toggle;
        }

        if keyboard_modifiers.shift() {
            return Self::Range;
        }

        Self::Single
    }
}

pub fn select_row_ids<'a, S: BuildHasher>(
    row_ids: impl Iterator<Item = &'a TableIdentifier> + Clone,
    current_selected_row_ids: &HashSet<&TableIdentifier, S>,
    target_row_id: &TableIdentifier,
    anchor_row_id: &TableIdentifier,
    select_operation: SelectOperation,
) -> (HashSet<TableIdentifier>, TableIdentifier) {
    if row_ids.clone().next().is_none() {
        return get_default_return(current_selected_row_ids, anchor_row_id);
    }

    let Some(target_row_index) = row_ids.clone().position(|row_id| row_id == target_row_id) else {
        return get_default_return(current_selected_row_ids, anchor_row_id);
    };

    let mut anchor_row_id = anchor_row_id;
    let anchor_row_index = row_ids
        .clone()
        .position(|row_id| row_id == anchor_row_id)
        .unwrap_or(0);

    if let Some(first_row_id) = row_ids.clone().next()
        && anchor_row_index == 0
        && first_row_id != anchor_row_id
    {
        anchor_row_id = first_row_id;
    }

    match select_operation {
        SelectOperation::Single => (
            HashSet::from_iter([target_row_id.to_owned()]),
            target_row_id.to_owned(),
        ),
        SelectOperation::Range => {
            let start_index = min(target_row_index, anchor_row_index);
            let end_index = max(target_row_index, anchor_row_index);

            (
                row_ids
                    .skip(start_index)
                    .take(end_index - start_index + 1)
                    .cloned()
                    .collect(),
                anchor_row_id.to_owned(),
            )
        }
        SelectOperation::Toggle => {
            if current_selected_row_ids.contains(target_row_id) {
                (
                    current_selected_row_ids
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
                        .chain(current_selected_row_ids.iter().copied().cloned())
                        .collect(),
                    target_row_id.to_owned(),
                )
            }
        }
        SelectOperation::Union => {
            let mut new_selected_row_ids: HashSet<TableIdentifier> =
                current_selected_row_ids.iter().copied().cloned().collect();

            let start_index = min(target_row_index, anchor_row_index);
            let end_index = max(target_row_index, anchor_row_index);

            new_selected_row_ids.extend(
                row_ids
                    .skip(start_index)
                    .take(end_index - start_index + 1)
                    .cloned(),
            );

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

    use crate::assert_matches;

    use super::*;

    fn get_iterator(input: &str) -> impl Iterator<Item = TableIdentifier> {
        input.chars().map(Into::<String>::into)
    }

    fn get_row_ids_source() -> Vec<TableIdentifier> {
        get_iterator("abcdefghijklmnopqrstuvwxyz").collect()
    }

    fn get_row_ids(row_ids: &[TableIdentifier]) -> impl Iterator<Item = &TableIdentifier> + Clone {
        row_ids.iter()
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
    fn should_get_correct_select_operation_from_keyboard_modifiers() {
        let keyboard_modifiers = keyboard::Modifiers::empty();
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(keyboard_modifiers),
            SelectOperation::Single
        );

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(keyboard_modifiers),
            SelectOperation::Toggle
        );

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(keyboard_modifiers),
            SelectOperation::Range
        );

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(keyboard_modifiers),
            SelectOperation::Union
        );
    }

    #[test]
    fn should_get_selected_rows_without_modifiers() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Single,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_without_modifiers_for_an_already_selected_row() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("a").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Single,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_without_modifiers_and_not_include_previously_selected_rows() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("cdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Single,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_select() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Toggle,
        );

        assert_selected_row_ids(new_selected_row_ids, "a");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_unselect() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("a").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "a".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Toggle,
        );

        assert_selected_row_ids(new_selected_row_ids, "");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_select_and_maintain_existing_selected_rows()
     {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("cdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Toggle,
        );

        assert_selected_row_ids(new_selected_row_ids, "acdefgh");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_unselect_and_maintain_existing_selected_rows()
     {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("acdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Toggle,
        );

        assert_selected_row_ids(new_selected_row_ids, "cdefgh");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Range,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select_with_already_selected_rows() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("cdefgh").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Range,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select_already_selected_row() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("c").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "c".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Range,
        );

        assert_selected_row_ids(new_selected_row_ids, "c");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_when_anchor_row_id_is_not_in_row_ids() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "c".to_owned();
        let anchor_row_id = "!".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Range,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_with_already_selected_rows() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("cdef").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
        );

        assert_selected_row_ids(new_selected_row_ids, "abcdef");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_already_selected_rows() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("abc").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_a_set_of_non_contiguous_rows()
     {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("wxyz").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "c".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
        );

        assert_selected_row_ids(new_selected_row_ids, "abcwxyz");
        assert_anchor_row_id(new_anchor_row_id, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_the_whole_table() {
        let row_ids_source = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids_source);

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "z".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
        );

        assert_selected_row_ids(new_selected_row_ids, &row_ids_source.join(""));
        assert_anchor_row_id(new_anchor_row_id, "z");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_the_whole_table_with_existing_selections()
     {
        let row_ids_source = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids_source);

        let selected_row_ids: Vec<String> = get_iterator("gdrfxz").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "a".to_owned();
        let anchor_row_id = "z".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
        );

        assert_selected_row_ids(new_selected_row_ids, &row_ids_source.join(""));
        assert_anchor_row_id(new_anchor_row_id, "z");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_when_anchor_row_id_is_not_in_row_ids()
     {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: HashSet<&String> = HashSet::new();

        let target_row_id = "c".to_owned();
        let anchor_row_id = "!".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
        );

        assert_selected_row_ids(new_selected_row_ids, "abc");
        assert_anchor_row_id(new_anchor_row_id, "a");
    }

    #[test]
    fn should_return_selected_rows_given_when_target_row_id_is_not_found_in_row_ids() {
        let row_ids = get_row_ids_source();
        let row_ids = get_row_ids(&row_ids);

        let selected_row_ids: Vec<String> = get_iterator("gdrfxz").collect();
        let selected_row_ids: HashSet<&String> = selected_row_ids.iter().collect();

        let target_row_id = "!".to_owned();
        let anchor_row_id = "z".to_owned();

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids,
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Union,
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

        let (new_selected_row_ids, new_anchor_row_id) = select_row_ids(
            row_ids.iter(),
            &selected_row_ids,
            &target_row_id,
            &anchor_row_id,
            SelectOperation::Single,
        );

        assert_selected_row_ids(new_selected_row_ids, "def");
        assert_anchor_row_id(new_anchor_row_id, "z");
    }
}
