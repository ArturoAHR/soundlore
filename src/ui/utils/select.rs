use std::{
    cmp::{max, min},
    hash::Hash,
    iter,
};

use iced::keyboard;
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub enum SelectOperation<'a, T> {
    Single {
        target_value: &'a T,
    },
    Toggle {
        target_value: &'a T,
    },
    Range {
        target_value: &'a T,
        anchor_value: Option<&'a T>,
    },
    Union {
        target_value: &'a T,
        anchor_value: Option<&'a T>,
    },
    All {
        anchor_value: Option<&'a T>,
    },
}

impl<'a, T> SelectOperation<'a, T> {
    pub fn from_keyboard_modifiers(
        keyboard_modifiers: keyboard::Modifiers,
        target_value: &'a T,
        anchor_value: Option<&'a T>,
    ) -> Self {
        if keyboard_modifiers.command() && keyboard_modifiers.shift() {
            return Self::Union {
                target_value,
                anchor_value,
            };
        }

        if keyboard_modifiers.command() {
            return Self::Toggle { target_value };
        }

        if keyboard_modifiers.shift() {
            return Self::Range {
                target_value,
                anchor_value,
            };
        }

        Self::Single { target_value }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn select_values<'a, T, C>(
    values: impl Iterator<Item = &'a T> + Clone,
    current_selected_values: impl Iterator<Item = &'a T> + Clone,
    select_operation: SelectOperation<'a, T>,
) -> (C, Option<T>)
where
    C: FromIterator<T>,
    T: Clone + PartialEq + Eq + Hash + 'a,
{
    if values.clone().next().is_none() {
        return get_default_return(current_selected_values, None);
    }

    match select_operation {
        SelectOperation::Single { target_value } => (
            iter::once(target_value.to_owned()).collect(),
            Some(target_value.to_owned()),
        ),
        SelectOperation::Range {
            target_value,
            anchor_value,
        } => {
            let Some(target_value_index) = values.clone().position(|value| value == target_value)
            else {
                return get_default_return(current_selected_values, anchor_value);
            };

            let (anchor_value, anchor_value_index) = get_anchor(values.clone(), anchor_value);

            let start_index = min(target_value_index, anchor_value_index);
            let end_index = max(target_value_index, anchor_value_index);

            (
                values
                    .skip(start_index)
                    .take(end_index - start_index + 1)
                    .cloned()
                    .collect(),
                Some(anchor_value.clone()),
            )
        }
        SelectOperation::Toggle { target_value } => {
            if current_selected_values
                .clone()
                .any(|selected_value| selected_value == target_value)
            {
                (
                    current_selected_values
                        .filter(|&row_id| row_id != target_value)
                        .cloned()
                        .collect(),
                    Some(target_value.to_owned()),
                )
            } else {
                (
                    iter::once(target_value.to_owned())
                        .chain(current_selected_values.cloned())
                        .collect(),
                    Some(target_value.to_owned()),
                )
            }
        }
        SelectOperation::Union {
            target_value,
            anchor_value,
        } => {
            let Some(target_value_index) = values.clone().position(|value| value == target_value)
            else {
                return get_default_return(current_selected_values, anchor_value);
            };

            let (anchor_value, anchor_value_index) = get_anchor(values.clone(), anchor_value);

            let mut new_selected_row_ids: FxHashSet<T> = current_selected_values.cloned().collect();

            let start_index = min(target_value_index, anchor_value_index);
            let end_index = max(target_value_index, anchor_value_index);

            new_selected_row_ids.extend(
                values
                    .skip(start_index)
                    .take(end_index - start_index + 1)
                    .cloned(),
            );

            (
                new_selected_row_ids.into_iter().collect(),
                Some(anchor_value.clone()),
            )
        }
        SelectOperation::All { anchor_value } => (values.cloned().collect(), anchor_value.cloned()),
    }
}

/// Gets the anchor value and anchor value index from given values
///
/// # Panics
/// If the values iterator given to this function is empty the function will panic
fn get_anchor<'a, T>(
    values: impl Iterator<Item = &'a T> + Clone,
    anchor_value: Option<&'a T>,
) -> (&'a T, usize)
where
    T: Clone + PartialEq + Eq + Hash + 'a,
{
    anchor_value
        .and_then(|anchor_value| {
            let anchor_value_index = values.clone().position(|value| value == anchor_value)?;

            Some((anchor_value, anchor_value_index))
        })
        .unwrap_or_else(|| {
            let first_value = values.take(1).next();

            first_value.map_or_else(
                || panic!("Cannot derive selection anchor value from empty value set"),
                |value| (value, 0),
            )
        })
}

fn get_default_return<'a, T, C>(
    current_selected_values: impl Iterator<Item = &'a T> + Clone,
    anchor_value: Option<&T>,
) -> (C, Option<T>)
where
    C: FromIterator<T>,
    T: Clone + PartialEq + Eq + Hash + 'a,
{
    (
        current_selected_values.cloned().collect(),
        anchor_value.cloned(),
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::assert_matches;

    use super::*;

    fn get_iterator(input: &str) -> impl Iterator<Item = String> {
        input.chars().map(Into::<String>::into)
    }

    fn get_values() -> Vec<String> {
        get_iterator("abcdefghijklmnopqrstuvwxyz").collect()
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assert_selected_values(selected_values: FxHashSet<String>, expected_selected_values: &str) {
        assert_eq!(
            selected_values.len(),
            expected_selected_values.len(),
            "Amount of selected values does not match the expected amount

        selected_values: {selected_values:?},
        expected_selected_values (string): \"{expected_selected_values}\"
            ",
        );

        let expected_values = get_iterator(expected_selected_values);

        for expected_value in expected_values {
            assert!(
                selected_values.contains(&expected_value),
                "Selected values set does not contain the following expected value: {expected_value}."
            );
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assert_anchor_value(anchor_value: Option<String>, expected_anchor_value: &str) {
        assert_eq!(anchor_value.unwrap(), expected_anchor_value.to_owned());
    }

    #[test]
    fn should_get_correct_select_operation_from_keyboard_modifiers() {
        let target_value = String::new();
        let anchor_value = None;

        let keyboard_modifiers = keyboard::Modifiers::empty();
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(
                keyboard_modifiers,
                &target_value,
                anchor_value.as_ref()
            ),
            SelectOperation::Single { target_value: _ }
        );

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(
                keyboard_modifiers,
                &target_value,
                anchor_value.as_ref()
            ),
            SelectOperation::Toggle { target_value: _ }
        );

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(
                keyboard_modifiers,
                &target_value,
                anchor_value.as_ref()
            ),
            SelectOperation::Range {
                target_value: _,
                anchor_value: _
            }
        );

        let mut keyboard_modifiers = keyboard::Modifiers::empty();
        keyboard_modifiers.insert(keyboard::Modifiers::SHIFT);
        keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);
        assert_matches!(
            SelectOperation::from_keyboard_modifiers(
                keyboard_modifiers,
                &target_value,
                anchor_value.as_ref()
            ),
            SelectOperation::Union {
                target_value: _,
                anchor_value: _
            }
        );
    }

    #[test]
    fn should_get_selected_rows_without_modifiers() {
        let values = get_values();

        let selected_values: FxHashSet<String> = FxHashSet::default();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Single {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "a");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_without_modifiers_for_an_already_selected_row() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("a").collect();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Single {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "a");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_without_modifiers_and_not_include_previously_selected_rows() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("cdefgh").collect();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Single {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "a");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_select() {
        let values = get_values();

        let selected_values: FxHashSet<String> = FxHashSet::default();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Toggle {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "a");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_unselect() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("a").collect();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Toggle {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_select_and_maintain_existing_selected_rows()
     {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("cdefgh").collect();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Toggle {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "acdefgh");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_modifier_to_unselect_and_maintain_existing_selected_rows()
     {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("acdefgh").collect();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Toggle {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "cdefgh");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select() {
        let values = get_values();

        let selected_values: FxHashSet<String> = FxHashSet::default();

        let target_value = "a".to_owned();
        let anchor_value = Some("c".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Range {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abc");
        assert_anchor_value(new_anchor_value, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select_with_already_selected_rows() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("cdefgh").collect();

        let target_value = "a".to_owned();
        let anchor_value = Some("c".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Range {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abc");
        assert_anchor_value(new_anchor_value, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_to_select_already_selected_row() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("c").collect();

        let target_value = "c".to_owned();
        let anchor_value = Some("c".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Range {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "c");
        assert_anchor_value(new_anchor_value, "c");
    }

    #[test]
    fn should_get_selected_rows_with_shift_modifier_when_anchor_row_id_is_not_in_row_ids() {
        let values = get_values();

        let selected_values: FxHashSet<String> = FxHashSet::default();

        let target_value = "c".to_owned();
        let anchor_value = Some("!".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Range {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abc");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier() {
        let values = get_values();

        let selected_values: FxHashSet<String> = FxHashSet::default();

        let target_value = "a".to_owned();
        let anchor_value = Some("c".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abc");
        assert_anchor_value(new_anchor_value, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_with_already_selected_rows() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("cdef").collect();

        let target_value = "a".to_owned();
        let anchor_value = Some("c".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abcdef");
        assert_anchor_value(new_anchor_value, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_already_selected_rows() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("abc").collect();

        let target_value = "a".to_owned();
        let anchor_value = Some("c".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abc");
        assert_anchor_value(new_anchor_value, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_a_set_of_non_contiguous_rows()
     {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("wxyz").collect();

        let target_value = "a".to_owned();
        let anchor_value = Some("c".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abcwxyz");
        assert_anchor_value(new_anchor_value, "c");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_the_whole_table() {
        let values = get_values();

        let selected_values: FxHashSet<String> = FxHashSet::default();

        let target_value = "a".to_owned();
        let anchor_value = Some("z".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, &values.join(""));
        assert_anchor_value(new_anchor_value, "z");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_selecting_the_whole_table_with_existing_selections()
     {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("gdrfxz").collect();

        let target_value = "a".to_owned();
        let anchor_value = Some("z".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, &values.join(""));
        assert_anchor_value(new_anchor_value, "z");
    }

    #[test]
    fn should_get_selected_rows_with_control_and_shift_modifier_when_anchor_row_id_is_not_in_row_ids()
     {
        let values = get_values();

        let selected_values: FxHashSet<String> = FxHashSet::default();

        let target_value = "c".to_owned();
        let anchor_value = Some("!".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abc");
        assert_anchor_value(new_anchor_value, "a");
    }

    #[test]
    fn should_return_selected_rows_given_when_target_row_id_is_not_found_in_row_ids() {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("gdrfxz").collect();

        let target_value = "!".to_owned();
        let anchor_value = Some("z".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Union {
                target_value: &target_value,
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "gdrfxz");
        assert_anchor_value(new_anchor_value, "z");
    }

    #[test]
    fn should_return_selected_rows_given_when_row_ids_is_empty() {
        let values = Vec::new();

        let selected_values: Vec<String> = get_iterator("def").collect();

        let target_value = "a".to_owned();

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::Single {
                target_value: &target_value,
            },
        );

        assert_selected_values(new_selected_values, "def");
        assert_matches!(new_anchor_value, None);
    }

    #[test]
    fn should_select_all_values_when_performing_select_all_operation() {
        let values = get_values();

        let selected_values: Vec<String> = Vec::new();

        let anchor_value = Some("z".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::All {
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abcdefghijklmnopqrstuvwxyz");
        assert_anchor_value(new_anchor_value, "z");
    }

    #[test]
    fn should_select_all_values_when_performing_select_all_operation_with_already_selected_values()
    {
        let values = get_values();

        let selected_values: Vec<String> = get_iterator("def").collect();

        let anchor_value = Some("z".to_owned());

        let (new_selected_values, new_anchor_value) = select_values(
            values.iter(),
            selected_values.iter(),
            SelectOperation::All {
                anchor_value: anchor_value.as_ref(),
            },
        );

        assert_selected_values(new_selected_values, "abcdefghijklmnopqrstuvwxyz");
        assert_anchor_value(new_anchor_value, "z");
    }
}
