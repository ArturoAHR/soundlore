use std::path::{Path, PathBuf};

use rfd::AsyncFileDialog;
use sqlx::SqlitePool;

use iced::{
    Element, Subscription, Task,
    time::{every, milliseconds},
    widget::{column, row},
};
use tracing::{error, info, instrument};

use crate::{
    app::Message::LoadTracks,
    error::AppError,
    library::scanner::scan_files_in_directory,
    playback::{
        PlaybackController, PlaybackControllerError, PlaybackControllerStatus,
        engine::device::watch_default_device, event::PlaybackControllerEvent,
        handler::PlaybackMessage,
    },
    track::{models::Track, repository::get_tracks},
    ui::{
        components::{
            explorer_pane::{self, ExplorerPane},
            main_pane::{self, MainPane},
            navigation_bar::{self, NavigationBar},
            playback_bar::{self, PlaybackBar, PlaybackBarViewContext},
            queue_pane::{self, QueuePane},
            status_bar::{self, StatusBar},
            track_information_pane::{self, TrackInformationPane},
        },
        theme::Theme,
    },
};

pub use crate::outcome::Outcome;

pub struct App {
    pub pool: SqlitePool,
    pub ui_scale: f32,
    pub theme: Theme,
    pub status: AppStatus,
    pub current_playing_track: Option<Track>,
    pub tracks: Vec<Track>,

    pub playback_controller: PlaybackController,

    pub navigation_bar: NavigationBar,
    pub explorer_pane: ExplorerPane,
    pub main_pane: MainPane,
    pub queue_pane: QueuePane,
    pub track_information_pane: TrackInformationPane,
    pub status_bar: StatusBar,
    pub playback_bar: PlaybackBar,
}

#[derive(Debug, PartialEq)]
pub enum AppStatus {
    Idle,
    // TODO: Add progress with count
    AddingTracks,
    // TODO: Add optional error data
    FinishedAddingTracks,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadTracks,
    LoadedTracks(Result<Vec<Track>, AppError>),
    ScanDirectory(Option<Vec<PathBuf>>),
    ScannedDirectory(Result<(), AppError>),

    NavigationBar(navigation_bar::Event),
    ExplorerPane(explorer_pane::Event),
    MainPane(main_pane::Event),
    QueuePane(queue_pane::Event),
    TrackInformationPane(track_information_pane::Event),
    StatusBar(status_bar::Event),
    PlaybackBar(playback_bar::Event),

    Playback(PlaybackMessage),
}

impl App {
    #[instrument(skip(pool, playback_controller))]
    pub fn new(
        pool: SqlitePool,
        theme: Theme,
        ui_scale: f32,
        playback_controller: PlaybackController,
    ) -> (Self, Task<Message>) {
        info!("Setting up App instance.");

        (
            App {
                pool,
                theme,
                ui_scale,
                status: AppStatus::Idle,
                tracks: Vec::new(),
                current_playing_track: None,

                playback_controller,

                navigation_bar: NavigationBar {},
                explorer_pane: ExplorerPane {},
                main_pane: MainPane {},
                queue_pane: QueuePane {},
                track_information_pane: TrackInformationPane {},
                status_bar: StatusBar {},
                playback_bar: PlaybackBar::new(),
            },
            Task::done(Message::LoadTracks),
        )
    }

    pub fn title(&self) -> String {
        String::from("Soundlore")
    }

    #[instrument(skip(self), level = "debug",
        fields(
            current_track = self.current_playing_track.as_ref().map(|track| {
                Path::new(&track.file_path)
                    .file_name()
                    .unwrap_or(track.file_path.as_ref())
                    .to_str()
            })
        )
    )]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadTracks => {
                let pool = self.pool.clone();

                Task::perform(async move { get_tracks(pool).await }, Message::LoadedTracks)
            }
            Message::LoadedTracks(tracks) => match tracks {
                Ok(tracks) => {
                    self.tracks = tracks;

                    Task::none()
                }
                Err(_) => Task::none(),
            },
            Message::ScanDirectory(Some(directories)) => {
                let pool = self.pool.clone();
                self.status = AppStatus::AddingTracks;

                Task::perform(
                    async move { scan_files_in_directory(pool, directories).await },
                    Message::ScannedDirectory,
                )
            }
            Message::ScanDirectory(None) => Task::none(),
            Message::ScannedDirectory(scan_result) => {
                let task = match scan_result {
                    Ok(_) => Task::done(LoadTracks),
                    Err(_) => Task::none(),
                };

                self.status = AppStatus::FinishedAddingTracks;

                task
            }
            Message::NavigationBar(event) => self.handle_navigation_bar(event),
            Message::ExplorerPane(event) => self.handle_explorer_pane(event),
            Message::MainPane(event) => self.handle_main_pane(event),
            Message::QueuePane(event) => self.handle_queue_pane(event),
            Message::TrackInformationPane(event) => self.handle_track_information_pane(event),
            Message::StatusBar(event) => self.handle_status_bar(event),
            Message::PlaybackBar(event) => self.handle_playback_bar(event),
            Message::Playback(event) => self.handle_playback(event),
        }
    }

    fn handle_navigation_bar(&mut self, event: navigation_bar::Event) -> Task<Message> {
        let (task, outcome) = self.navigation_bar.update(event);
        let component_task = task.map(Message::NavigationBar);

        let Some(outcome) = outcome else {
            return component_task;
        };
        let outcome_task = match outcome {
            navigation_bar::Outcome::OpenSelectDirectoryDialog => Task::perform(
                async {
                    AsyncFileDialog::new()
                        .pick_folders()
                        .await
                        .map(|handles| handles.iter().map(|handle| handle.path().into()).collect())
                },
                Message::ScanDirectory,
            ),
        };

        Task::batch([outcome_task, component_task])
    }

    fn handle_explorer_pane(&mut self, event: explorer_pane::Event) -> Task<Message> {
        let (task, outcome) = self.explorer_pane.update(event);
        let component_task = task.map(Message::ExplorerPane);

        let Some(outcome) = outcome else {
            return component_task;
        };

        let outcome_task = match outcome {};

        Task::batch([outcome_task, component_task])
    }

    fn handle_queue_pane(&mut self, event: queue_pane::Event) -> Task<Message> {
        let (task, outcome) = self.queue_pane.update(event);
        let component_task = task.map(Message::QueuePane);

        let Some(outcome) = outcome else {
            return component_task;
        };

        let outcome_task = match outcome {};

        Task::batch([outcome_task, component_task])
    }

    fn handle_track_information_pane(
        &mut self,
        event: track_information_pane::Event,
    ) -> Task<Message> {
        let (task, outcome) = self.track_information_pane.update(event);
        let component_task = task.map(Message::TrackInformationPane);

        let Some(outcome) = outcome else {
            return component_task;
        };

        let outcome_task = match outcome {};

        Task::batch([outcome_task, component_task])
    }

    fn handle_status_bar(&mut self, event: status_bar::Event) -> Task<Message> {
        let (task, outcome) = self.status_bar.update(event);
        let component_task = task.map(Message::StatusBar);

        let Some(outcome) = outcome else {
            return component_task;
        };

        let outcome_task = match outcome {};

        Task::batch([outcome_task, component_task])
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        let navigation_bar = self
            .navigation_bar
            .view(&self.theme)
            .map(Message::NavigationBar);

        let explorer_pane = self
            .explorer_pane
            .view(&self.theme)
            .map(Message::ExplorerPane);

        let main_pane = self
            .main_pane
            .view(&self.theme, &self.tracks)
            .map(Message::MainPane);

        let queue_pane = self.queue_pane.view(&self.theme).map(Message::QueuePane);

        let track_information_pane = self
            .track_information_pane
            .view(&self.theme)
            .map(Message::TrackInformationPane);

        let status_bar = self
            .status_bar
            .view(&self.theme, &self.status)
            .map(Message::StatusBar);

        let playback_bar_context = PlaybackBarViewContext {
            current_playing_track: &self.current_playing_track,
            theme: &self.theme,
        };

        let playback_bar = self
            .playback_bar
            .view(playback_bar_context)
            .map(Message::PlaybackBar);

        column![
            navigation_bar,
            row![
                explorer_pane,
                main_pane,
                column![queue_pane, track_information_pane]
            ],
            status_bar,
            playback_bar
        ]
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![Subscription::run(watch_default_device)];

        subscriptions.push(
            every(milliseconds(16)).map(|_| Message::Playback(PlaybackMessage::PollPlaybackEvent)),
        );

        Subscription::batch(subscriptions)
    }

    pub fn scale_factor(&self) -> f32 {
        self.ui_scale
    }

    pub fn theme(&self) -> Theme {
        self.theme.to_owned()
    }
}
