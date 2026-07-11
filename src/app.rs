use std::path::{Path, PathBuf};

use iced_split::{horizontal_split, vertical_split};
use sqlx::SqlitePool;

use iced::{
    Element, Length, Size, Subscription, Task,
    time::{every, milliseconds},
    widget::{column, container},
    window,
};
use tracing::{error, info, instrument};

use crate::{
    app::Message::LoadTracks,
    constants::{MIN_HORIZONTAL_SPLIT_PANE_HEIGHT, MIN_VERTICAL_SPLIT_PANE_WIDTH},
    error::AppError,
    library::scanner::scan_files_in_directory,
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
        utils::pane::{are_pane_heights_valid, are_pane_widths_valid},
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

    pub window_size: Size,
    pub main_window_id: Option<window::Id>,

    pub pane_split_ratio: PaneSplitPositions,

    pub playback_controller: PlaybackController,

    pub navigation_bar: NavigationBar,
    pub explorer_pane: ExplorerPane,
    pub main_pane: MainPane,
    pub queue_pane: QueuePane,
    pub track_information_pane: TrackInformationPane,
    pub status_bar: StatusBar,
    pub playback_bar: PlaybackBar,
}

#[derive(Debug)]
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
    SplitDragged(PaneSplit, f64),
    WindowResized(Option<window::Id>, Size),
    GetWindowId(window::Id),

    NavigationBar(navigation_bar::Message),
    ExplorerPane(explorer_pane::Message),
    MainPane(main_pane::Message),
    QueuePane(queue_pane::Message),
    TrackInformationPane(track_information_pane::Message),
    StatusBar(status_bar::Message),
    PlaybackBar(playback_bar::Message),

    Playback(playback::Message),
}

pub struct PaneSplitPositions {
    pub explorer_main: f64,
    pub main_queue: f64,
    pub queue_track_information: f64,
}

#[derive(Debug, Clone)]
pub enum PaneSplit {
    /// The split between the explorer pane and main pane.
    ExplorerMain,
    /// The split between the main pane and the column with the queue pane and the track information pane.
    MainQueue,
    /// The split between the queue pane and the track information pane.
    QueueTrackInformation,
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
            Self {
                pool,
                theme,
                ui_scale,
                status: AppStatus::Idle,
                tracks: Vec::new(),
                current_playing_track: None,

                window_size: Size::default(),
                main_window_id: None,

                pane_split_ratio: PaneSplitPositions {
                    explorer_main: 0.2,
                    main_queue: 0.7,
                    queue_track_information: 0.8,
                },

                playback_controller,

                navigation_bar: NavigationBar {},
                explorer_pane: ExplorerPane {},
                main_pane: MainPane {},
                queue_pane: QueuePane {},
                track_information_pane: TrackInformationPane {},
                status_bar: StatusBar {},
                playback_bar: PlaybackBar::new(),
            },
            Task::batch([
                Task::done(Message::LoadTracks),
                window::latest().and_then(|window_id| {
                    Task::batch([
                        Task::done(Message::GetWindowId(window_id)),
                        window::size(window_id).map(|size| Message::WindowResized(None, size)),
                    ])
                }),
            ]),
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
                    .unwrap_or_else(|| track.file_path.as_ref())
                    .to_str()
            })
        )
    )]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let mut task = Task::none();

        match message {
            Message::SplitDragged(split, split_ratio) => {
                match split {
                    PaneSplit::ExplorerMain => {
                        // Since the main-queue split is a children of the explorer-main split, we
                        // need to calculate the new ratio of the main-queue split so the split stays
                        // in place.
                        let main_queue_split_ratio = 1.0
                            - (1.0 - self.pane_split_ratio.explorer_main)
                                * (1.0 - self.pane_split_ratio.main_queue)
                                / (1.0 - split_ratio);

                        if are_pane_widths_valid(
                            split_ratio,
                            main_queue_split_ratio,
                            self.window_size.width as f64,
                            MIN_VERTICAL_SPLIT_PANE_WIDTH,
                        ) {
                            self.pane_split_ratio.explorer_main = split_ratio;
                            self.pane_split_ratio.main_queue = main_queue_split_ratio;
                        }
                    }
                    PaneSplit::MainQueue => {
                        if are_pane_widths_valid(
                            self.pane_split_ratio.explorer_main,
                            split_ratio,
                            self.window_size.width as f64,
                            MIN_VERTICAL_SPLIT_PANE_WIDTH,
                        ) {
                            self.pane_split_ratio.main_queue = split_ratio;
                        }
                    }
                    PaneSplit::QueueTrackInformation => {
                        if are_pane_heights_valid(
                            split_ratio,
                            self.window_size.height as f64,
                            MIN_HORIZONTAL_SPLIT_PANE_HEIGHT,
                        ) {
                            self.pane_split_ratio.queue_track_information = split_ratio;
                        }
                    }
                }
            }
            Message::WindowResized(window_id, size) => {
                if window_id.is_none() || window_id == self.main_window_id {
                    self.window_size = size;
                }
            }
            Message::GetWindowId(window_id) => self.main_window_id = Some(window_id),

            Message::LoadTracks => {
                let pool = self.pool.clone();

                task = Task::perform(async move { get_tracks(pool).await }, Message::LoadedTracks);
            }
            Message::LoadedTracks(tracks) => match tracks {
                Ok(tracks) => {
                    self.tracks = tracks;
                }
                Err(error) => error!("Failed to load tracks: {error}"),
            },
            Message::ScanDirectory(Some(directories)) => {
                let pool = self.pool.clone();
                self.status = AppStatus::AddingTracks;

                task = Task::perform(
                    async move { scan_files_in_directory(pool, directories).await },
                    Message::ScannedDirectory,
                );
            }
            Message::ScanDirectory(None) => {}
            Message::ScannedDirectory(scan_result) => {
                task = match scan_result {
                    Ok(()) => Task::done(LoadTracks),
                    Err(_) => Task::none(),
                };

                self.status = AppStatus::FinishedAddingTracks;
            }

            Message::NavigationBar(event) => task = self.handle_navigation_bar(event),
            Message::ExplorerPane(event) => task = self.handle_explorer_pane(event),
            Message::MainPane(event) => task = self.handle_main_pane(event),
            Message::QueuePane(event) => task = self.handle_queue_pane(event),
            Message::TrackInformationPane(event) => {
                task = self.handle_track_information_pane(event);
            }
            Message::StatusBar(event) => task = self.handle_status_bar(event),
            Message::PlaybackBar(event) => task = self.handle_playback_bar(event),
            Message::Playback(event) => task = self.handle_playback(event),
        }

        task
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        let navigation_bar = self.view_navigation_bar();

        let explorer_pane = self.view_explorer_pane();

        let main_pane = self.view_main_pane();

        let queue_pane = self.view_queue_pane();

        let track_information_pane = self.view_track_information_pane();

        let status_bar = self.view_status_bar();

        let playback_bar = self.view_playback_bar();

        let queue_track_information_pane_split = horizontal_split(
            queue_pane,
            track_information_pane,
            self.pane_split_ratio.queue_track_information as f32,
            |split_at| Message::SplitDragged(PaneSplit::QueueTrackInformation, split_at as f64),
        )
        .handle_width(5.0);

        let main_queue_pane_split = vertical_split(
            main_pane,
            queue_track_information_pane_split,
            self.pane_split_ratio.main_queue as f32,
            |split_at| Message::SplitDragged(PaneSplit::MainQueue, split_at as f64),
        )
        .handle_width(5.0);

        let explorer_main_pane_split = vertical_split(
            explorer_pane,
            main_queue_pane_split,
            self.pane_split_ratio.explorer_main as f32,
            |split_at| Message::SplitDragged(PaneSplit::ExplorerMain, split_at as f64),
        )
        .handle_width(5.0);

        column![
            navigation_bar,
            container(explorer_main_pane_split)
                .height(Length::Fill)
                .width(Length::Fill),
            status_bar,
            playback_bar
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![Subscription::run(watch_default_device)];

        subscriptions.push(
            every(milliseconds(16))
                .map(|_| Message::Playback(playback::Message::PollPlaybackEvent)),
        );

        subscriptions.push(
            window::resize_events()
                .map(|(window_id, size)| Message::WindowResized(Some(window_id), size)),
        );

        Subscription::batch(subscriptions)
    }

    pub fn scale_factor(&self) -> f32 {
        self.ui_scale
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
