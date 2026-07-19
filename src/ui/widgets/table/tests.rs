use std::time::Instant;

use iced::{Event, Point, Rectangle, Settings, Size, advanced::mouse, widget::text, window};
use iced_palace::widget::ellipsized_text;
use iced_test::{
    Simulator,
    simulator::{Snapshot, click},
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
const HEADER_HEIGHT: f32 = 35.0;
const SCROLL_WIDTH: f32 = 12.0;
const TEST_ROW_COUNT: f32 = 10000.0;

struct TestApp {
    rows: Vec<TestData>,
    selected_rows: HashSet<TableIdentifier>,
}

#[derive(Debug, Clone)]
enum TestMessage {
    RowSelected(HashSet<TableIdentifier>),
    RowDoubleClicked(TableIdentifier),
    ColumnHeaderCellClicked(TableIdentifier),
}

#[derive(Debug)]
struct TestData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub data: String,
}

impl Identifiable for TestData {
    fn id(&self) -> &TableIdentifier {
        &self.id
    }
}

impl TestApp {
    pub fn new() -> Self {
        Self {
            rows: (0..TEST_ROW_COUNT as usize)
                .map(|index| TestData {
                    id: (index + 1).to_string(),
                    name: format!("Test Name {}", index + 1),
                    description: format!("Test Description {}", index + 1),
                    data: format!("Test Data {}", index + 1),
                })
                .collect(),
            selected_rows: HashSet::new(),
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
            column(
                "playing-indicator".to_owned(),
                None,
                |_test_data: &TestData| Space::new(),
            )
            .width(50.0),
            column(
                "test-name".to_owned(),
                Some(text("Test Name").into()),
                |test_data: &TestData| ellipsized_text(test_data.name.clone()),
            )
            .resizable(true),
            column(
                "test-description".to_owned(),
                Some(text("Test Description").into()),
                |test_data: &TestData| ellipsized_text(test_data.description.clone()),
            )
            .resizable(true),
            column(
                "test-data-1".to_owned(),
                Some(text("Test Data 1").into()),
                |test_data: &TestData| ellipsized_text(test_data.data.clone()),
            )
            .resizable(true),
            column(
                "test-data-2".to_owned(),
                Some(text("Test Data 2").into()),
                |test_data: &TestData| ellipsized_text(test_data.data.clone()),
            )
            .resizable(true),
        ];

        table(columns, &self.rows)
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

fn get_middle_of_table_grid_y() -> f32 {
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
        get_middle_of_table_grid_y(),
        get_clickable_row_height(visible_row_number, scroll_offset),
    );

    ui.point_at(click_position);

    ui.simulate(click());
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

    ui.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: target_scroll_thumb_bounds.center(),
    })]);

    ui.simulate([Event::Window(
        window::Event::RedrawRequested(Instant::now()),
    )]);

    ui.point_at(target_scroll_thumb_bounds.center());
    ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    Some(target_scroll_offset)
}

fn assert_row_selection_message(message: &TestMessage, expected_selected_rows: &[TableIdentifier]) {
    match message {
        TestMessage::RowSelected(selected_rows) => {
            for selected_row_id in expected_selected_rows {
                assert!(
                    selected_rows.contains(selected_row_id),
                    "Selected rows do not contain expected row id:\n    selected_rows: {selected_rows:?}\n    selected_row_id: {selected_row_id:?}\n"
                );
            }
        }
        _ => unreachable!("Received unexpected events: {message:?}"),
    }
}

fn assert_row_double_clicking_message(
    message: &TestMessage,
    expected_selected_rows: &[TableIdentifier],
    double_clicked_row_id: &TableIdentifier,
) {
    match message {
        TestMessage::RowSelected(selected_rows) => {
            for selected_row_id in expected_selected_rows {
                assert!(
                    selected_rows.contains(selected_row_id),
                    "Selected rows do not contain expected row id:\n    selected_rows: {selected_rows:?}\n    selected_row_id: {selected_row_id:?}\n"
                );
            }
        }
        TestMessage::RowDoubleClicked(row_id) => {
            assert_eq!(double_clicked_row_id, row_id);
        }
        TestMessage::ColumnHeaderCellClicked(_) => {
            unreachable!("Received unexpected events: {message:?}")
        }
    }
}

fn get_snapshot(ui: &mut Simulator<'_, TestMessage, Theme, iced::Renderer>) -> Snapshot {
    ui.snapshot(&Theme::default()).unwrap()
}

#[test]
fn should_select_first_row() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    click_row_position(&mut ui, 0, 0.0);

    let first_row_id = app.rows[0].id().clone();
    let expected_selected_rows = vec![first_row_id];

    let snapshot = get_snapshot(&mut ui);

    assert_snapshot(&snapshot);

    for message in ui.into_messages() {
        assert_row_selection_message(&message, &expected_selected_rows);

        app.update(message);
    }
}

#[test]
fn should_double_click_first_row() {
    let mut app = TestApp::new();

    let mut ui = simulator(&app);

    click_row_position(&mut ui, 0, 0.0);
    click_row_position(&mut ui, 0, 0.0);

    let first_row_id = app.rows[0].id().clone();
    let expected_selected_rows = vec![first_row_id.clone()];

    let snapshot = get_snapshot(&mut ui);

    assert_snapshot(&snapshot);

    for message in ui.into_messages() {
        assert_row_double_clicking_message(&message, &expected_selected_rows, &first_row_id);

        app.update(message);
    }
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

    let selected_row_id = app.rows[row_number - 1].id().clone();
    let expected_selected_rows = vec![selected_row_id.clone()];

    let snapshot = get_snapshot(&mut ui);

    assert_snapshot(&snapshot);

    for message in ui.into_messages() {
        assert_row_double_clicking_message(&message, &expected_selected_rows, &selected_row_id);

        app.update(message);
    }
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
