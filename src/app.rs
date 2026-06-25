use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use iced::{
    Element, Subscription, Task,
    time::{every, milliseconds},
    widget::{column, row},
};
use tracing::{info, instrument};

use crate::{
    app::Event::LoadTracks,
    error::AppError,
    library::scanner::scan_files_in_directory,
    message::Message,
    playback::{self, PlaybackController, engine::device::watch_default_device},
    track::{models::Track, repository::get_tracks},
    ui::{
        components::{
            explorer_pane::{self, ExplorerPane},
            main_pane::{self, MainPane},
            navigation_bar::{self, NavigationBar},
            playback_bar::{self, PlaybackBar},
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
pub enum Event {
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

    Playback(playback::Event),
}

impl App {
    #[instrument(skip(pool, playback_controller))]
    pub fn new(
        pool: SqlitePool,
        theme: Theme,
        ui_scale: f32,
        playback_controller: PlaybackController,
    ) -> (Self, Task<Message<Event>>) {
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
            Task::done(Message::new(Event::LoadTracks)),
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
    pub fn update(&mut self, message: Message<Event>) -> Task<Message<Event>> {
        match message.payload {
            Event::LoadTracks => {
                let pool = self.pool.clone();

                message.task_from(async move { get_tracks(pool).await }, Event::LoadedTracks)
            }
            Event::LoadedTracks(tracks) => match tracks {
                Ok(tracks) => {
                    self.tracks = tracks;

                    Task::none()
                }
                Err(_) => Task::none(),
            },
            Event::ScanDirectory(Some(ref directories)) => {
                let pool = self.pool.clone();
                self.status = AppStatus::AddingTracks;

                let directories = directories.clone();
                message.task_from(
                    async move { scan_files_in_directory(pool, directories).await },
                    Event::ScannedDirectory,
                )
            }
            Event::ScanDirectory(None) => Task::none(),
            Event::ScannedDirectory(ref scan_result) => {
                let task = match scan_result {
                    Ok(_) => Task::done(message.new_from(LoadTracks)),
                    Err(_) => Task::none(),
                };

                self.status = AppStatus::FinishedAddingTracks;

                task
            }
            Event::NavigationBar(ref event) => {
                self.handle_navigation_bar(message.new_from(event.clone()))
            }
            Event::ExplorerPane(ref event) => {
                self.handle_explorer_pane(message.new_from(event.clone()))
            }
            Event::MainPane(ref event) => self.handle_main_pane(message.new_from(event.clone())),
            Event::QueuePane(ref event) => self.handle_queue_pane(message.new_from(event.clone())),
            Event::TrackInformationPane(ref event) => {
                self.handle_track_information_pane(message.new_from(event.clone()))
            }
            Event::StatusBar(ref event) => self.handle_status_bar(message.new_from(event.clone())),
            Event::PlaybackBar(ref event) => {
                self.handle_playback_bar(message.new_from(event.clone()))
            }
            Event::Playback(ref event) => self.handle_playback(message.new_from(event.clone())),
        }
    }

    pub fn view(&self) -> Element<'_, Message<Event>, Theme> {
        let navigation_bar = self.view_navigation_bar();

        let explorer_pane = self.view_explorer_pane();

        let main_pane = self.view_main_pane();

        let queue_pane = self.view_queue_pane();

        let track_information_pane = self.view_track_information_pane();

        let status_bar = self.view_status_bar();

        let playback_bar = self.view_playback_bar();

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

    pub fn subscription(&self) -> Subscription<Message<Event>> {
        let mut subscriptions = vec![Subscription::run(watch_default_device)];

        subscriptions.push(
            every(milliseconds(16))
                .map(|_| Message::new(Event::Playback(playback::Event::PollPlaybackEvent))),
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
