use std::{iter, slice, time::Instant};

use iced::{
    Event, Point, Rectangle, Settings, Size,
    advanced::mouse,
    keyboard::{self},
    widget::text,
    window,
};
use iced_palace::widget::ellipsized_text;
use iced_test::{
    Simulator,
    simulator::{Snapshot, click, press_key},
};
use pretty_assertions::assert_eq;

use crate::{
    test::snapshot::assert_snapshot,
    ui::{
        theme::Theme,
        widgets::table::{
            bounds::{get_effective_scroll_area_bounds, get_table_scroll_bounds},
            scroll::get_scroll_thumb_bounds,
        },
    },
};

use super::*;

const APP_SIZE: Size = Size::new(1000.0, 1000.0);
const BOUNDS: Rectangle = Rectangle::new(Point { x: 0.0, y: 0.0 }, APP_SIZE);
const ROW_HEIGHT: f32 = 30.0;
const COLUMN_WIDTHS: [f32; 5] = [48.0, 238.0, 238.0, 238.0, 238.0];
const HEADER_HEIGHT: f32 = 35.0;
const SCROLL_WIDTH: f32 = 12.0;
const TEST_ROW_COUNT: f32 = 10000.0;

struct TestApp {
    rows: Vec<TestData>,
    selected_rows: FxHashSet<<TestData as Identifiable>::Identifier>,
}

#[derive(Debug, Clone)]
enum TestMessage {
    RowSelected(FxHashSet<<TestData as Identifiable>::Identifier>),
    RowDoubleClicked(<TestData as Identifiable>::Identifier),
    ColumnHeaderCellClicked(i64),
}

#[derive(Debug, Clone)]
struct TestData {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub data: String,
}

impl Identifiable for TestData {
    type Identifier = usize;

    fn id(&self) -> &Self::Identifier {
        &self.id
    }
}

impl TableRow for TestData {
    fn header_row_id() -> Self::Identifier {
        usize::MAX
    }
}

impl TestApp {
    pub fn new() -> Self {
        Self {
            rows: (0..TEST_ROW_COUNT as usize)
                .map(|index| TestData {
                    id: index + 1,
                    name: format!("Test Name {}", index + 1),
                    description: format!("Test Description {}", index + 1),
                    data: format!("Test Data {}", index + 1),
                })
                .collect(),
            selected_rows: FxHashSet::default(),
        }
    }

    pub fn update(&mut self, message: TestMessage) {
        match message {
            TestMessage::RowSelected(selected_rows) => {
                self.selected_rows = selected_rows;
            }
            TestMessage::RowDoubleClicked(_) | TestMessage::ColumnHeaderCellClicked(_) => {}
        }
    }

    pub fn view(&self) -> Element<'_, TestMessage, Theme> {
        let columns = vec![
            column(1, None, |_test_data: &TestData| Space::new()).width(COLUMN_WIDTHS[0]),
            column(2, Some(text("Test Name").into()), |test_data: &TestData| {
                ellipsized_text(test_data.name.clone())
            })
            .width(COLUMN_WIDTHS[1])
            .resizable(true),
            column(
                3,
                Some(text("Test Description").into()),
                |test_data: &TestData| ellipsized_text(test_data.description.clone()),
            )
            .width(COLUMN_WIDTHS[2])
            .resizable(true),
            column(
                4,
                Some(text("Test Data 1").into()),
                |test_data: &TestData| ellipsized_text(test_data.data.clone()),
            )
            .width(COLUMN_WIDTHS[3])
            .resizable(true),
            column(
                5,
                Some(text("Test Data 2").into()),
                |test_data: &TestData| ellipsized_text(test_data.data.clone()),
            )
            .width(COLUMN_WIDTHS[4])
            .resizable(true),
        ];

        table(columns, self.rows.iter().collect())
            .selected_rows(&self.selected_rows)
            .row_height(ROW_HEIGHT)
            .header_height(HEADER_HEIGHT)
            .scroll_width(SCROLL_WIDTH)
            .on_row_select(TestMessage::RowSelected)
            .on_row_double_click(TestMessage::RowDoubleClicked)
            .on_header_cell_click(TestMessage::ColumnHeaderCellClicked)
            .into()
    }
}

fn simulator(app: &TestApp) -> Simulator<'_, TestMessage, Theme, iced::Renderer> {
    Simulator::with_size(Settings::default(), APP_SIZE, app.view())
}

fn point_and_click(
    ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>,
    click_position: Point,
) {
    ui.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: click_position,
    })]);

    ui.point_at(click_position);

    ui.simulate(click());
}

fn point_and_mouse_left_button_press(
    ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>,
    click_position: Point,
) {
    ui.point_at(click_position);

    ui.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: click_position,
    })]);

    ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);
}

fn point_and_mouse_left_button_release(
    ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>,
    click_position: Point,
) {
    ui.point_at(click_position);

    ui.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: click_position,
    })]);

    ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
}

fn get_middle_of_headers_row_y() -> f32 {
    HEADER_HEIGHT / 2.0
}

fn get_middle_of_header_cell_x(column_number: usize) -> f32 {
    COLUMN_WIDTHS
        .iter()
        .copied()
        .take(column_number)
        .sum::<f32>()
        - (COLUMN_WIDTHS[column_number - 1] / 2.0)
}

fn get_middle_of_table_grid_x() -> f32 {
    (APP_SIZE.width - SCROLL_WIDTH) / 2.0
}

fn get_clickable_row_height(visible_row_number: usize, scroll_offset: f32) -> f32 {
    let row_offset_start = HEADER_HEIGHT - ROW_HEIGHT * (scroll_offset / ROW_HEIGHT).fract();

    (row_offset_start + ROW_HEIGHT * visible_row_number as f32)
        .clamp(HEADER_HEIGHT + 1.0, APP_SIZE.height)
}

fn click_row_position(
    ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>,
    visible_row_number: usize,
    scroll_offset: f32,
) {
    let click_position = Point::new(
        get_middle_of_table_grid_x(),
        get_clickable_row_height(visible_row_number, scroll_offset),
    );

    point_and_click(ui, click_position);
}

fn click_row_and_drag(
    ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>,
    starting_visible_row_number: usize,
    ending_visible_row_number: usize,
    scroll_offset: f32,
) {
    let start_position = Point::new(
        get_middle_of_table_grid_x(),
        get_clickable_row_height(starting_visible_row_number, scroll_offset),
    );

    let end_position = Point::new(
        get_middle_of_table_grid_x(),
        get_clickable_row_height(ending_visible_row_number, scroll_offset),
    );

    point_and_mouse_left_button_press(ui, start_position);

    point_and_mouse_left_button_release(ui, end_position);
}

fn click_header_position(
    ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>,
    column_number: usize,
) {
    let click_position = Point::new(
        get_middle_of_header_cell_x(column_number),
        get_middle_of_headers_row_y(),
    );

    point_and_click(ui, click_position);
}

fn scroll_to_row_number(
    ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>,
    row_number: usize,
    scroll_offset: f32,
) -> Option<f32> {
    let total_scrollable_content_length = TEST_ROW_COUNT * ROW_HEIGHT;
    let target_scroll_offset = ((row_number - 1) as f32 * ROW_HEIGHT)
        .min(total_scrollable_content_length - (APP_SIZE.height - HEADER_HEIGHT));

    let scroll_bounds = get_table_scroll_bounds(BOUNDS, SCROLL_WIDTH);
    let effective_scroll_area_bounds =
        get_effective_scroll_area_bounds(scroll_bounds, HEADER_HEIGHT);
    let scroll_thumb_bounds = get_scroll_thumb_bounds(
        effective_scroll_area_bounds,
        total_scrollable_content_length,
        scroll_offset,
    )?;
    let target_scroll_thumb_bounds = get_scroll_thumb_bounds(
        effective_scroll_area_bounds,
        total_scrollable_content_length,
        target_scroll_offset,
    )?;

    ui.point_at(scroll_thumb_bounds.center());
    ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    ui.point_at(target_scroll_thumb_bounds.center());
    ui.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: target_scroll_thumb_bounds.center(),
    })]);

    ui.simulate([Event::Window(
        window::Event::RedrawRequested(Instant::now()),
    )]);

    ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    Some(target_scroll_offset)
}

fn select_all_rows(ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>) {
    let keyboard_modifiers = keyboard::Modifiers::COMMAND;

    ui.simulate([Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard_modifiers,
    ))]);

    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modified_key,
        physical_key,
        location,
        text,
        repeat,
        ..
    }) = press_key(keyboard::Key::Character("a".into()), None)
    else {
        panic!("")
    };

    ui.simulate([Event::Keyboard(keyboard::Event::KeyPressed {
        modifiers: keyboard_modifiers,
        key,
        modified_key,
        physical_key,
        location,
        text,
        repeat,
    })]);
}

fn focus(ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>) {
    ui.point_at(Point {
        x: APP_SIZE.width - SCROLL_WIDTH / 2.0, // Empty space in the scroll bar that is no-op to click
        y: 0.0,
    });
    ui.simulate(click());
}

fn blur(ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>) {
    ui.point_at(Point {
        x: APP_SIZE.width + 1.0,
        y: APP_SIZE.height + 1.0,
    });
    ui.simulate(click());
}

fn assert_row_selection_message(
    message: &TestMessage,
    expected_selected_rows: &[<TestData as Identifiable>::Identifier],
) {
    match message {
        TestMessage::RowSelected(selected_rows) => {
            assert_eq!(
                selected_rows.len(),
                expected_selected_rows.len(),
                "    selected_rows: {selected_rows:?}\n    expected_selected_rows: {expected_selected_rows:?}\n"
            );

            for selected_row_id in expected_selected_rows {
                assert!(
                    selected_rows.contains(selected_row_id),
                    "Selected rows do not contain expected row id:\n    selected_rows: {selected_rows:?}\n    selected_row_id: {selected_row_id:?}\n    expected_selected_rows: {expected_selected_rows:?}\n"
                );
            }
        }
        _ => unreachable!("Received unexpected events: {message:?}"),
    }
}

fn assert_row_double_clicking_message(
    message: &TestMessage,
    double_clicked_row_id: <TestData as Identifiable>::Identifier,
) {
    match message {
        TestMessage::RowDoubleClicked(row_id) => {
            assert_eq!(double_clicked_row_id, *row_id);
        }
        _ => unreachable!("Received unexpected events: {message:?}"),
    }
}

fn assert_header_clicking_message(message: &TestMessage, expected_column_id: i64) {
    match message {
        TestMessage::ColumnHeaderCellClicked(column_id) => {
            assert_eq!(expected_column_id, *column_id);
        }
        _ => {
            unreachable!("Received unexpected events: {message:?}")
        }
    }
}

fn get_snapshot(ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>) -> Snapshot {
    ui.snapshot(&Theme::default()).unwrap()
}

fn snapshot_and_assert(app: &TestApp) {
    let mut ui = simulator(app);

    let snapshot = get_snapshot(&mut ui);

    assert_snapshot(&snapshot);
}

#[test]
fn should_select_first_row() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    click_row_position(&mut ui, 1, 0.0);

    let first_row_id = app.rows[0].id();

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], &[*first_row_id]);
    app.update(messages[0].clone());
    assert_eq!(messages.len(), 1);

    snapshot_and_assert(&app);
}

#[test]
fn should_select_second_row_after_selecting_first_one() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    click_row_position(&mut ui, 1, 0.0);
    click_row_position(&mut ui, 2, 0.0);

    let first_row_id = *app.rows[0].id();
    let second_row_id = *app.rows[1].id();

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], &[first_row_id]);
    app.update(messages[0].clone());
    assert_row_selection_message(&messages[1], &[second_row_id]);
    app.update(messages[1].clone());
    assert_eq!(messages.len(), 2);
}

#[test]
fn should_double_click_first_row() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    click_row_position(&mut ui, 1, 0.0);
    click_row_position(&mut ui, 1, 0.0);

    let first_row_id = *app.rows[0].id();

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], slice::from_ref(&first_row_id));
    app.update(messages[0].clone());
    assert_row_selection_message(&messages[1], slice::from_ref(&first_row_id));
    app.update(messages[1].clone());
    assert_row_double_clicking_message(&messages[2], first_row_id);
    app.update(messages[2].clone());
    assert_eq!(messages.len(), 3);

    snapshot_and_assert(&app);
}

#[test]
fn should_click_second_header() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    let column_number = 2;
    let expected_column_id = 2;
    click_header_position(&mut ui, column_number);

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_header_clicking_message(&messages[0], expected_column_id);
    app.update(messages[0].clone());

    snapshot_and_assert(&app);
}

#[test]
fn should_toggle_select_rows() {
    let mut app = TestApp::new();
    let second_row_id = *app.rows[1].id();
    app.selected_rows = iter::once(second_row_id).collect();

    let mut ui = simulator(&app);

    let keyboard_modifiers = keyboard::Modifiers::COMMAND;
    ui.simulate([Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard_modifiers,
    ))]);

    click_row_position(&mut ui, 1, 0.0);

    let first_row_id = *app.rows[0].id();

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], &[first_row_id, second_row_id]);
    app.update(messages[0].clone());
    assert_eq!(messages.len(), 1);

    snapshot_and_assert(&app);
}

#[test]
fn should_toggle_already_selected_rows() {
    let mut app = TestApp::new();
    let second_row_id = *app.rows[1].id();
    app.selected_rows = iter::once(second_row_id).collect();

    let mut ui = simulator(&app);

    let keyboard_modifiers = keyboard::Modifiers::COMMAND;
    ui.simulate([Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard_modifiers,
    ))]);

    click_row_position(&mut ui, 2, 0.0);

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], &[]);
    app.update(messages[0].clone());
    assert_eq!(messages.len(), 1);

    snapshot_and_assert(&app);
}

#[test]
fn should_range_select_already_selected_rows() {
    let mut app = TestApp::new();
    let seventh_row_id = *app.rows[6].id();
    app.selected_rows = iter::once(seventh_row_id).collect();

    let mut ui = simulator(&app);

    let keyboard_modifiers = keyboard::Modifiers::SHIFT;
    ui.simulate([Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard_modifiers,
    ))]);

    click_row_position(&mut ui, 5, 0.0);

    let expected_selected_row_ids: Vec<<TestData as Identifiable>::Identifier> = app
        .rows
        .iter()
        .take(5)
        .map(Identifiable::id)
        .copied()
        .collect();

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], &expected_selected_row_ids);
    app.update(messages[0].clone());
    assert_eq!(messages.len(), 1);

    snapshot_and_assert(&app);
}

#[test]
fn should_union_select_already_selected_rows() {
    let mut app = TestApp::new();
    let seventh_row_id = *app.rows[6].id();
    app.selected_rows = iter::once(seventh_row_id).collect();

    let mut ui = simulator(&app);

    let mut keyboard_modifiers = keyboard::Modifiers::SHIFT;
    keyboard_modifiers.insert(keyboard::Modifiers::COMMAND);
    ui.simulate([Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard_modifiers,
    ))]);

    click_row_position(&mut ui, 5, 0.0);

    let expected_selected_row_ids: Vec<<TestData as Identifiable>::Identifier> = app
        .rows
        .iter()
        .take(5)
        .map(Identifiable::id)
        .copied()
        .chain([seventh_row_id])
        .collect();

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], &expected_selected_row_ids);
    app.update(messages[0].clone());
    assert_eq!(messages.len(), 1);

    snapshot_and_assert(&app);
}

#[test]
fn should_scroll_to_and_select_table_row_at_the_half_of_the_table() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    let row_number = TEST_ROW_COUNT as usize / 2;

    let scroll_offset = scroll_to_row_number(&mut ui, row_number, 0.0)
        .unwrap_or_else(|| panic!("Could not scroll to row number: {row_number}"));

    click_row_position(&mut ui, 0, scroll_offset);
    click_row_position(&mut ui, 0, scroll_offset);

    let selected_row_id = *app.rows[row_number - 1].id();

    let snapshot = get_snapshot(&mut ui);

    assert_snapshot(&snapshot);

    let messages: Vec<TestMessage> = ui.into_messages().collect();
    assert_row_selection_message(&messages[0], slice::from_ref(&selected_row_id));
    app.update(messages[0].clone());
    assert_row_selection_message(&messages[1], slice::from_ref(&selected_row_id));
    app.update(messages[1].clone());
    assert_row_double_clicking_message(&messages[2], selected_row_id);
    app.update(messages[2].clone());
    assert_eq!(messages.len(), 3);
}

#[test]
fn should_scroll_to_the_end_of_the_table() {
    let app = TestApp::new();

    let mut ui = simulator(&app);

    let _scroll_offset = scroll_to_row_number(&mut ui, TEST_ROW_COUNT as usize, 0.0)
        .unwrap_or_else(|| panic!("Could not scroll to the end of the table"));

    let snapshot = get_snapshot(&mut ui);

    assert_snapshot(&snapshot);
}

#[test]
fn should_scroll_to_the_end_of_the_table_with_mouse_wheel() {
    let app = TestApp::new();

    let mut ui = simulator(&app);

    ui.point_at(Point { x: 0.0, y: 0.0 });
    ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines {
            x: 0.0,
            y: -TEST_ROW_COUNT * ROW_HEIGHT / 15.0,
        },
    })]);

    let snapshot = get_snapshot(&mut ui);

    assert_snapshot(&snapshot);
}

#[test]
fn should_drag_and_select_rows() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    let starting_row_number = 2;
    let ending_row_number = 7;

    click_row_and_drag(&mut ui, starting_row_number, ending_row_number, 0.0);

    let expected_selected_rows: Vec<<TestData as Identifiable>::Identifier> = app.rows
        [starting_row_number - 1..ending_row_number]
        .iter()
        .map(Identifiable::id)
        .copied()
        .collect();

    let messages: Vec<TestMessage> = ui.into_messages().collect();

    assert_row_selection_message(&messages[0], &[expected_selected_rows[0]]);
    app.update(messages[0].clone());
    assert_row_selection_message(&messages[1], &expected_selected_rows);
    app.update(messages[1].clone());

    snapshot_and_assert(&app);
}

#[test]
fn should_select_all() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    focus(&mut ui);
    select_all_rows(&mut ui);

    let messages: Vec<TestMessage> = ui.into_messages().collect();

    let expected_selected_rows: Vec<<TestData as Identifiable>::Identifier> =
        app.rows.iter().map(Identifiable::id).copied().collect();

    assert_row_selection_message(&messages[0], &expected_selected_rows);
    app.update(messages[0].clone());

    snapshot_and_assert(&app);
}

#[test]
fn should_not_select_all_if_widget_is_blurred() {
    let app = TestApp::new();

    let mut ui = simulator(&app);

    blur(&mut ui);
    select_all_rows(&mut ui);

    assert_eq!(
        ui.into_messages().count(),
        0,
        "There shouldn't have been any emitted message."
    );
}
