use std::{cell::Cell, collections::VecDeque};

use thiserror::Error;
use tracing::instrument;

use crate::{
    constants::PLAYBACK_QUEUE_LENGTH,
    playback::queue::entry::{PlaybackQueueEntry, PlaybackQueueEntryId, PlaybackQueueEntrySource},
    track::models::TrackId,
};

pub mod entry;
mod generation;
pub mod handler;

#[cfg(test)]
pub mod tests;

pub use handler::Message;

#[derive(Debug, Error, Clone)]
pub enum PlaybackQueueError {
    #[error("Attempted to remove queued track beyond queue length")]
    InvalidQueueRemovePosition,
}

#[derive(Debug, Clone)]
pub struct PlaybackQueue {
    /// Incrementing unique identifier granted to each queue entry upon creation.
    next_entry_id: Cell<PlaybackQueueEntryId>,

    /// Current position in the queue.
    pub cursor: usize,

    /// Contains the ids of the tracks that are in the queue.
    pub entries: VecDeque<PlaybackQueueEntry>,

    /// Contains the list of track ids from which the queue can pick from, it assumes the list is non-repeating.
    track_pool: Vec<TrackId>,

    pub repeat_mode: PlaybackRepeatMode,
    pub order: PlaybackQueueOrder,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            next_entry_id: Cell::default(),
            cursor: 0,
            entries: VecDeque::new(),
            track_pool: Vec::new(),
            repeat_mode: PlaybackRepeatMode::NoRepeat,
            order: PlaybackQueueOrder::Sequential,
        }
    }

    fn get_next_entry_id(&self) -> PlaybackQueueEntryId {
        let id = self.next_entry_id.get();

        self.next_entry_id.replace(id + 1);

        id
    }

    /// Sets the track pool and optionally the initial track.
    pub fn start(&mut self, track_pool: Vec<TrackId>, playing_track_id: Option<TrackId>) {
        self.cursor = 0;
        self.entries.clear();
        self.track_pool = track_pool;

        if let Some(playing_track_id) = playing_track_id {
            let id = self.get_next_entry_id();

            self.entries
                .push_back(PlaybackQueueEntry::system(id, playing_track_id));
        }

        self.generate_next_entries(PLAYBACK_QUEUE_LENGTH);
    }

    #[instrument(skip(self), ret)]
    pub fn current(&self) -> Option<TrackId> {
        self.entries.get(self.cursor).map(|entry| entry.track_id)
    }

    /// Peeks the next track without moving the cursor
    pub fn peek_next(&mut self) -> Option<TrackId> {
        let original_queue_length = self.entries.len();

        if self.get_remaining_tracks() <= PLAYBACK_QUEUE_LENGTH {
            self.generate_next_entries(PLAYBACK_QUEUE_LENGTH);
        }

        self.entries
            .get(self.cursor + original_queue_length.min(1))
            .map(|entry| entry.track_id)
    }

    /// Peeks the previous track without moving the cursor
    pub fn peek_previous(&mut self) -> Option<TrackId> {
        let original_queue_length = self.entries.len();

        if self.cursor == 0 {
            self.generate_previous_entries(PLAYBACK_QUEUE_LENGTH);
        }

        let previous_track_index = self.cursor.checked_sub(original_queue_length.min(1))?;
        self.entries
            .get(previous_track_index)
            .map(|entry| entry.track_id)
    }

    #[allow(clippy::should_implement_trait)]
    #[instrument(skip(self), ret)]
    /// Gets the next track and moves the cursor forward, generating new entries if needed.
    pub fn next(&mut self) -> Option<TrackId> {
        if self.get_remaining_tracks() <= PLAYBACK_QUEUE_LENGTH {
            self.generate_next_entries(PLAYBACK_QUEUE_LENGTH);
        }

        let next_track_id = self
            .entries
            .get(self.cursor + 1)
            .map(|entry| entry.track_id);

        if next_track_id.is_some() {
            self.cursor += 1;
        }

        next_track_id
    }

    #[instrument(skip(self), ret)]
    /// Gets the previous track and moves the cursor backwards, generating new previous entries if applicable.
    pub fn previous(&mut self) -> Option<TrackId> {
        if self.cursor == 0 {
            self.generate_previous_entries(PLAYBACK_QUEUE_LENGTH);
        }

        let previous_track_index = self.cursor.checked_sub(1)?;
        let previous_track_id = self
            .entries
            .get(previous_track_index)
            .map(|entry| entry.track_id);

        if previous_track_id.is_some() {
            self.cursor = self.cursor.saturating_sub(1);
        }

        previous_track_id
    }

    /// Inserts a track id in the queue at the given index, if the index is out of bounds it
    /// adds the track id at the end.
    pub fn insert(&mut self, index: usize, track_id: TrackId) {
        let insert_index = index.min(self.entries.len());

        let id = self.get_next_entry_id();

        self.entries
            .insert(insert_index, PlaybackQueueEntry::user(id, track_id));

        if insert_index <= self.cursor {
            self.cursor = (self.cursor + 1).min(self.entries.len().saturating_sub(1));
        }
    }

    /// Inserts a queue entry at the end of of a consecutive set of user entries starting at the entry after
    /// the cursor.
    pub fn insert_next(&mut self, track_id: TrackId) {
        let insert_index = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches!(entry.source, PlaybackQueueEntrySource::User))
            .fold(self.cursor, |current_insert_index, (index, _)| {
                if current_insert_index + 1 == index {
                    current_insert_index.max(index)
                } else {
                    current_insert_index
                }
            });

        self.insert(insert_index + 1, track_id);
    }

    /// Removes a track id from the queue at the given index, bear in mind this index
    /// can differ from the displayed queue pane table row number, to perform removal relative
    /// to the displayed tracks in the queue pane, use `remove_relative_to_cursor`
    ///
    /// # Errors
    /// This method will fail if the index is out of bounds.
    pub fn remove(&mut self, index: usize) -> Result<(), PlaybackQueueError> {
        if index >= self.entries.len() {
            return Err(PlaybackQueueError::InvalidQueueRemovePosition);
        }

        self.entries.remove(index);

        if index < self.cursor {
            self.cursor = self.cursor.saturating_sub(1);
        }

        // Guards against cursor falling outside of queue entries.
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));

        if self.get_remaining_tracks() <= PLAYBACK_QUEUE_LENGTH {
            self.generate_next_entries(PLAYBACK_QUEUE_LENGTH);
        }

        Ok(())
    }

    /// Removes the track id relative to the cursor, which current value determines the first
    /// track displayed in the queue pane table.
    pub fn remove_relative_to_cursor(&mut self, index: usize) -> Result<(), PlaybackQueueError> {
        self.remove(self.cursor + index)
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.entries.len().saturating_sub(1));

        if self.get_remaining_tracks() <= PLAYBACK_QUEUE_LENGTH {
            self.generate_next_entries(PLAYBACK_QUEUE_LENGTH);
        }
    }

    /// Removes all upcoming system entries from the queue
    pub fn truncate(&mut self) {
        let cursor_position = self.cursor.min(self.entries.len().saturating_add(1));
        let mut entries_tail = self
            .entries
            .split_off((cursor_position + 1).min(self.entries.len()));

        entries_tail.retain(|entry| matches!(entry.source, PlaybackQueueEntrySource::User));

        self.entries.extend(entries_tail);
    }

    /// Removes all tracks except the current one and the unplayed entries inserted by the user.
    pub fn prune(&mut self) {
        let cursor_position = self.cursor.min(self.entries.len());
        let mut entries_tail = self.entries.split_off(cursor_position);

        let entry_id = entries_tail.front().map(|entry| entry.id);
        entries_tail.retain(|entry| {
            entry_id.is_some_and(|current_entry_id| current_entry_id == entry.id)
                || matches!(entry.source, PlaybackQueueEntrySource::User)
        });

        self.entries = entries_tail;
        self.cursor = 0;
    }

    pub fn get_remaining_tracks(&self) -> usize {
        self.entries.len().saturating_sub(self.cursor)
    }

    pub fn cycle_queue_order(&mut self) {
        self.order = self.order.next();

        self.prune();
        self.generate_next_entries(PLAYBACK_QUEUE_LENGTH);
    }

    pub fn cycle_repeat_mode(&mut self) {
        let previous_repeat_mode = self.repeat_mode;

        self.repeat_mode = self.repeat_mode.next();

        if previous_repeat_mode.is_repeating() != self.repeat_mode.is_repeating() {
            match self.order {
                PlaybackQueueOrder::Sequential => {
                    self.prune();
                }
                PlaybackQueueOrder::Shuffle => {
                    self.truncate();
                }
            }
        }

        if self.get_remaining_tracks() <= PLAYBACK_QUEUE_LENGTH {
            self.generate_next_entries(PLAYBACK_QUEUE_LENGTH);
        }
    }
}

impl Default for PlaybackQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlaybackRepeatMode {
    NoRepeat,
    Repeat,
    RepeatOne,
}

impl PlaybackRepeatMode {
    pub fn is_repeating(&self) -> bool {
        matches!(self, Self::RepeatOne | Self::Repeat)
    }

    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::NoRepeat => Self::Repeat,
            Self::Repeat => Self::RepeatOne,
            Self::RepeatOne => Self::NoRepeat,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlaybackQueueOrder {
    Sequential,
    Shuffle,
}

impl PlaybackQueueOrder {
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::Sequential => Self::Shuffle,
            Self::Shuffle => Self::Sequential,
        }
    }
}
