use iced::{
    Element, Length, Renderer, Task, alignment,
    widget::{container, row, text},
};
use iced_palace::widget::ellipsized_text;
use tracing::instrument;

use crate::{
    app::AppStatus,
    event::Event,
    track::models::TrackId,
    ui::theme::{Theme, catalog},
};

pub mod handler;

#[derive(Debug)]
pub struct StatusBar {}

#[derive(Debug, Clone)]
#[expect(clippy::empty_enums, reason = "placeholder")]
pub enum Message {}

#[derive(Debug, Clone)]
#[expect(clippy::empty_enums, reason = "placeholder")]
pub enum Outcome {}

#[derive(Debug)]
pub struct StatusBarUpdateContext {}

impl StatusBar {
    #[instrument(skip(self), level = "debug")]
    pub fn update(&mut self, event: Message) -> (Task<Message>, Vec<Outcome>) {
        (Task::none(), vec![])
    }

    #[instrument(skip(self), level = "debug")]
    pub fn on_event(&mut self, event: &Event) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(
        &'a self,
        theme: &Theme,
        displayed_track_ids: &[TrackId],
        status: &AppStatus,
    ) -> Element<'a, Message, Theme, Renderer> {
        let status_label = match status {
            AppStatus::Idle => "",
            AppStatus::AddingTracks => "Adding tracks",
            AppStatus::FinishedAddingTracks => "Finished adding tracks",
        };

        let displayed_track_list_label = format!("{} tracks", displayed_track_ids.len());

        container(
            row![
                container(
                    ellipsized_text(status_label)
                        .size(theme.sizes.font.body)
                        .color(theme.palette.text_muted),
                )
                .width(Length::Fill),
                text(displayed_track_list_label)
                    .size(theme.sizes.font.body)
                    .color(theme.palette.text_muted),
            ]
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .height(Length::Fixed(theme.sizes.component.status_bar_height))
        .width(Length::Fill)
        .align_y(alignment::Vertical::Center)
        .padding([0.0, theme.sizes.space.md])
        .style(catalog::container::background_surface_sunken)
        .into()
    }
}
