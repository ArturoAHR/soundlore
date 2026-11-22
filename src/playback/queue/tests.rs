use pretty_assertions::assert_eq;
use rustc_hash::FxHashSet;

use crate::assert_matches;

use super::*;

fn generate_queue(
    track_pool: Vec<TrackId>,
    currently_playing_track_id: Option<TrackId>,
    repeat_mode: PlaybackRepeatMode,
    order: PlaybackQueueOrder,
) -> PlaybackQueue {
    let mut queue = PlaybackQueue::new();

    queue.start(track_pool, currently_playing_track_id);

    match currently_playing_track_id {
        Some(track_id) => {
            assert_eq!(queue.current(), Some(track_id));
        }
        None => {
            assert_eq!(queue.current(), Some(queue.track_pool[0]));
        }
    }

    match repeat_mode {
        PlaybackRepeatMode::NoRepeat => {
            assert_matches!(queue.repeat_mode, PlaybackRepeatMode::NoRepeat);
        }
        PlaybackRepeatMode::Repeat => {
            queue.cycle_repeat_mode();
            assert_matches!(queue.repeat_mode, PlaybackRepeatMode::Repeat);
        }
        PlaybackRepeatMode::RepeatOne => {
            queue.cycle_repeat_mode();
            queue.cycle_repeat_mode();

            assert_matches!(queue.repeat_mode, PlaybackRepeatMode::RepeatOne);
        }
    }

    match order {
        PlaybackQueueOrder::Sequential => {
            assert_matches!(queue.order, PlaybackQueueOrder::Sequential);
        }
        PlaybackQueueOrder::Shuffle => {
            queue.cycle_queue_order();

            assert_matches!(queue.order, PlaybackQueueOrder::Shuffle);
        }
    }

    for (index, track_id) in queue.track_pool.iter().enumerate() {
        assert_eq!(
            index, *track_id as usize,
            "The queue track pool is missing the expected value {index}"
        );
    }

    queue
}

#[test]
fn should_start_queue() {
    generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );
}

#[test]
fn should_start_queue_without_a_starting_track() {
    generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );
}

#[test]
fn should_play_the_track_pool_sequentially() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    for index in 1..10 {
        let next_track_id = queue
            .next()
            .unwrap_or_else(|| panic!("Queue unexpectedly ended when retrieving track id {index}"));

        assert_eq!(next_track_id, index, "Unexpected track queued");
    }

    let value_outside_queue = queue.next();

    assert!(
        value_outside_queue.is_none(),
        "Returned another track after the non-repeating queue ended: {value_outside_queue:?}"
    );
}

#[test]
fn should_play_the_track_pool_sequentially_back_and_forth() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    for index in 1..10 {
        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving next track id {index}")
        });

        assert_eq!(next_track_id, index, "Unexpected track queued");
    }

    let next_track = queue.next();

    assert!(
        next_track.is_none(),
        "Returned another track after the non-repeating queue ended: {next_track:?}"
    );

    for index in (0..9).rev() {
        let previous = queue.previous().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving previous track id {index}")
        });

        assert_eq!(previous, index, "Unexpected track queued");
    }

    let previous_track = queue.previous();

    assert!(
        previous_track.is_none(),
        "Returned another track after reaching the non-repeating queue start: {previous_track:?}"
    );
}

#[test]
fn should_play_the_track_pool_sequentially_repeating() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (1..10).chain(0..10) {
        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            next_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_play_the_track_pool_sequentially_repeating_backwards() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (0..10).chain(0..10).rev() {
        let previous_track_id = queue.previous().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            previous_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_play_the_track_pool_sequentially_in_repeat_one_mode() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (1..10).chain(0..10) {
        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            next_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_play_the_track_pool_sequentially_in_repeat_one_mode_backwards() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (0..10).chain(0..10).rev() {
        let previous_track_id = queue.previous().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            previous_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_peek_the_next_track_in_the_queue_sequentially() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    let next_track_id = queue.peek_next();

    assert_matches!(next_track_id, Some(1));
}

#[test]
fn should_peek_the_previous_track_in_the_queue_sequentially() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    let previous_track_id = queue.peek_previous();

    assert_matches!(previous_track_id, None);
}

#[test]
fn should_peek_the_next_track_in_the_queue_repeating_sequentially() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    let next_track_id = queue.peek_next();

    assert_matches!(next_track_id, Some(1));
}

#[test]
fn should_peek_the_previous_track_in_the_queue_repeating_sequentially() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    let previous_track_id = queue.peek_previous();

    assert_matches!(previous_track_id, Some(9));
}

#[test]
fn should_peek_the_next_track_in_the_queue_repeating_one_sequentially() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    let next_track_id = queue.peek_next();

    assert_matches!(next_track_id, Some(1));
}

#[test]
fn should_peek_the_previous_track_in_the_queue_repeating_one_sequentially() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    let previous_track_id = queue.peek_previous();

    assert_matches!(previous_track_id, Some(9));
}

#[test]
fn should_play_the_track_pool_sequentially_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    for index in 1..10 {
        let next_track_id = queue
            .next()
            .unwrap_or_else(|| panic!("Queue unexpectedly ended when retrieving track id {index}"));

        assert_eq!(next_track_id, index, "Unexpected track queued");
    }

    let value_outside_queue = queue.next();

    assert!(
        value_outside_queue.is_none(),
        "Returned another track after the non-repeating queue ended: {value_outside_queue:?}"
    );
}

#[test]
fn should_play_the_track_pool_sequentially_back_and_forth_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    for index in 1..10 {
        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving next track id {index}")
        });

        assert_eq!(next_track_id, index, "Unexpected track queued");
    }

    let next_track = queue.next();

    assert!(
        next_track.is_none(),
        "Returned another track after the non-repeating queue ended: {next_track:?}"
    );

    for index in (0..9).rev() {
        let previous = queue.previous().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving previous track id {index}")
        });

        assert_eq!(previous, index, "Unexpected track queued");
    }

    let previous_track = queue.previous();

    assert!(
        previous_track.is_none(),
        "Returned another track after reaching the non-repeating queue start: {previous_track:?}"
    );
}

#[test]
fn should_play_the_track_pool_sequentially_repeating_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (1..10).chain(0..10) {
        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            next_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_play_the_track_pool_sequentially_repeating_backwards_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (0..10).chain(0..10).rev() {
        let previous_track_id = queue.previous().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            previous_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_play_the_track_pool_sequentially_in_repeat_one_mode_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (1..10).chain(0..10) {
        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            next_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_play_the_track_pool_sequentially_in_repeat_one_mode_backwards_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    for expected_track_id in (0..10).chain(0..10).rev() {
        let previous_track_id = queue.previous().unwrap_or_else(|| {
            panic!("Queue unexpectedly ended when retrieving track id {expected_track_id}")
        });

        assert_eq!(
            previous_track_id, expected_track_id,
            "Unexpected track queued at queue position: {}",
            queue.cursor
        );
    }
}

#[test]
fn should_peek_the_next_track_in_the_queue_sequentially_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    let next_track_id = queue.peek_next();

    assert_matches!(next_track_id, Some(1));
}

#[test]
fn should_peek_the_previous_track_in_the_queue_sequentially_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    let previous_track_id = queue.peek_previous();

    assert_matches!(previous_track_id, None);
}

#[test]
fn should_peek_the_next_track_in_the_queue_repeating_sequentially_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    let next_track_id = queue.peek_next();

    assert_matches!(next_track_id, Some(1));
}

#[test]
fn should_peek_the_previous_track_in_the_queue_repeating_sequentially_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    let previous_track_id = queue.peek_previous();

    assert_matches!(previous_track_id, Some(9));
}

#[test]
fn should_peek_the_next_track_in_the_queue_repeating_one_sequentially_without_a_starting_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    let next_track_id = queue.peek_next();

    assert_matches!(next_track_id, Some(1));
}

#[test]
fn should_peek_the_previous_track_in_the_queue_repeating_one_sequentially_without_a_starting_track()
{
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::RepeatOne,
        PlaybackQueueOrder::Sequential,
    );

    let previous_track_id = queue.peek_previous();

    assert_matches!(previous_track_id, Some(9));
}

#[test]
fn should_queue_track_next() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.insert_next(10);

    assert_eq!(queue.next(), Some(10));
}

#[test]
fn should_queue_several_tracks_next() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.insert_next(10);
    queue.insert_next(11);
    queue.insert_next(12);

    assert_eq!(queue.next(), Some(10));
    assert_eq!(queue.next(), Some(11));
    assert_eq!(queue.next(), Some(12));
}

#[test]
fn should_queue_a_track_5_tracks_later() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.insert(queue.cursor + 5, 10);

    for _ in 0..4 {
        queue.next();
    }

    assert_eq!(queue.next(), Some(10));
}

#[test]
fn should_queue_insertion_should_happen_at_the_end_if_insert_index_exceeds_queue_entries_length() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    let original_queue_entries_length = queue.entries.len();

    queue.insert(queue.entries.len() + 10, 10);

    for _ in 0..original_queue_entries_length - 1 {
        queue.next();
    }

    assert_eq!(queue.next(), Some(10));
}

#[test]
fn should_queue_several_tracks_next_after_inserting_one_at_the_end() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.insert(queue.entries.len(), 15);

    queue.insert_next(10);
    queue.insert_next(11);
    queue.insert_next(12);

    assert_eq!(queue.next(), Some(10));
    assert_eq!(queue.next(), Some(11));
    assert_eq!(queue.next(), Some(12));
}

#[test]
fn should_set_cursor() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    // sixth element of the third loop
    queue.set_cursor(25);

    assert_eq!(queue.current(), Some(5));
}

#[test]
fn should_set_cursor_at_the_end_if_index_is_out_of_bounds_and_generate_more_entries() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    // sixth element of the third loop
    queue.set_cursor(queue.entries.len() + 1);

    assert_eq!(
        queue.current(),
        Some(queue.entries[queue.entries.len() - 1].track_id)
    );
    assert_eq!(queue.cursor, 50);
    assert_eq!(queue.entries.len(), 101);
}

#[test]
fn should_remove_the_next_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.remove(queue.cursor + 1).unwrap();

    assert_eq!(queue.next(), Some(2));
}

#[test]
fn should_remove_the_next_track_relative_to_cursor() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.remove_relative_to_cursor(1).unwrap();

    assert_eq!(queue.next(), Some(2));
}

#[test]
fn should_remove_the_current_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.remove(queue.cursor).unwrap();

    assert_eq!(queue.current(), Some(1));
}

#[test]
fn should_remove_relative_to_cursor_the_current_track() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.remove_relative_to_cursor(0).unwrap();

    assert_eq!(queue.current(), Some(1));
}

#[test]
fn should_remove_the_only_track_in_the_queue_and_restart_the_queue() {
    let mut queue = generate_queue(
        (0..10).collect(),
        Some(0),
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.set_cursor(queue.entries.len() - 1);

    queue.prune();

    queue.remove(queue.cursor).unwrap();

    assert_eq!(queue.current(), Some(0));
}

#[test]
fn should_remove_entry_behind_cursor() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    // sixth element of the third loop
    queue.set_cursor(25);

    queue.remove(0).unwrap();

    assert_eq!(queue.current(), Some(5));
    assert_eq!(queue.cursor, 24);
}

#[test]
fn should_fail_to_remove_when_index_is_out_of_bounds() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    let result = queue.remove(queue.entries.len());

    assert_matches!(result, Err(PlaybackQueueError::InvalidQueueRemovePosition));
}

#[test]
fn should_truncate() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.truncate();

    assert_eq!(queue.current(), Some(0));
    assert_eq!(queue.entries.len(), 1);
}

#[test]
fn should_truncate_an_empty_queue() {
    let mut queue = PlaybackQueue::new();

    queue.truncate();

    assert_eq!(queue.current(), None);
    assert_eq!(queue.entries.len(), 0);
}

#[test]
fn should_truncate_without_removing_future_user_queued_entries() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.set_cursor(25);

    queue.insert(queue.entries.len(), 15);
    queue.insert(0, 16);

    queue.insert_next(10);
    queue.insert_next(11);
    queue.insert_next(12);

    queue.truncate();

    assert_eq!(queue.peek_previous(), Some(4));
    assert_eq!(queue.current(), Some(5));
    assert_eq!(queue.next(), Some(10));
    assert_eq!(queue.next(), Some(11));
    assert_eq!(queue.next(), Some(12));
    assert_eq!(queue.next(), Some(15));
    assert_eq!(queue.entries.len(), PLAYBACK_QUEUE_LENGTH + 5 + 26);
}

#[test]
fn should_prune() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.set_cursor(25);

    queue.prune();

    assert_eq!(queue.current(), Some(5));
    assert_eq!(queue.entries.len(), 1);
}

#[test]
fn should_prune_an_empty_queue() {
    let mut queue = PlaybackQueue::new();

    queue.prune();

    assert_eq!(queue.current(), None);
    assert_eq!(queue.entries.len(), 0);
}

#[test]
fn should_prune_without_removing_future_user_queued_entries() {
    let mut queue = generate_queue(
        (0..10).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.set_cursor(25);

    queue.insert(queue.entries.len(), 15);
    queue.insert(0, 16);

    queue.insert_next(10);
    queue.insert_next(11);
    queue.insert_next(12);

    queue.prune();

    assert_eq!(queue.peek_previous(), Some(4));
    assert_eq!(queue.current(), Some(5));
    assert_eq!(queue.next(), Some(10));
    assert_eq!(queue.next(), Some(11));
    assert_eq!(queue.next(), Some(12));
    assert_eq!(queue.next(), Some(15));
    assert_eq!(queue.entries.len(), PLAYBACK_QUEUE_LENGTH * 2 + 5);
}

#[test]
fn should_not_repeat_tracks_when_shuffling() {
    for _ in 0..25 {
        let mut queue = generate_queue(
            (0..1000).collect(),
            Some(0),
            PlaybackRepeatMode::NoRepeat,
            PlaybackQueueOrder::Shuffle,
        );

        let mut played_tracks = FxHashSet::default();

        for index in 0..queue.track_pool.len() - 1 {
            let next_track_id = queue.next().unwrap_or_else(|| {
                panic!("Shuffled queue ran out of tracks before it should at index {index}")
            });

            assert!(
                !played_tracks.contains(&next_track_id),
                "Repeated track id: {next_track_id}"
            );
            played_tracks.insert(next_track_id);
        }

        assert_eq!(queue.next(), None);
    }
}

#[test]
fn should_not_repeat_tracks_when_shuffling_with_repeats_until_the_track_has_not_been_played_for_half_of_the_track_pool_length()
 {
    let mut queue = generate_queue(
        (0..1000).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Shuffle,
    );

    let mut played_tracks = FxHashSet::default();

    let no_repeat_threshold = queue.track_pool.len() / 2;

    for index in 0..queue.track_pool.len() * 10 {
        if index >= no_repeat_threshold {
            let track_id_that_can_repeat = queue.entries[index - no_repeat_threshold];

            played_tracks.remove(&track_id_that_can_repeat.track_id);
        }

        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Repeated Shuffled queue ran out of tracks at index {index}")
        });

        assert!(
            !played_tracks.contains(&next_track_id),
            "Repeated track id: {next_track_id}"
        );
        played_tracks.insert(next_track_id);
    }
}

#[test]
fn should_transition_to_sequential_order_properly() {
    let mut queue = generate_queue(
        (0..100).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Shuffle,
    );

    let next_shuffled_track_id = queue.next().unwrap();

    queue.cycle_queue_order();

    for index in (0..next_shuffled_track_id).rev() {
        let previous_sequential_track_id = queue.previous().unwrap();

        assert_eq!(index, previous_sequential_track_id);
    }

    assert_eq!(queue.current(), Some(0));

    for index in 1..queue.entries.len() - 1 {
        let next_sequential_track_id = queue.next().unwrap();

        assert_eq!(index, next_sequential_track_id as usize);
    }
}

#[test]
fn should_transition_to_shuffled_order_properly() {
    let mut queue = generate_queue(
        (0..100).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    queue.next().unwrap();

    queue.cycle_queue_order();

    // Previous tracks generation is not supported for shuffle.
    assert_eq!(queue.peek_previous(), None);

    let mut played_tracks = FxHashSet::default();

    for index in 0..queue.track_pool.len() - 1 {
        let next_track_id = queue.next().unwrap_or_else(|| {
            panic!("Shuffled queue ran out of tracks before it should at index {index}")
        });

        assert!(
            !played_tracks.contains(&next_track_id),
            "Repeated track id: {next_track_id}"
        );
        played_tracks.insert(next_track_id);
    }
}

#[test]
fn should_transition_to_a_repeating_repeat_mode_properly_with_sequential_order() {
    let mut queue = generate_queue(
        (0..100).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Sequential,
    );

    for _ in 0..queue.track_pool.len() - 1 {
        queue.next().unwrap();
    }

    assert_eq!(queue.next(), None);

    assert_eq!(queue.entries.len(), queue.track_pool.len());
    queue.cycle_repeat_mode();
    assert_eq!(
        queue.entries.len(),
        PLAYBACK_QUEUE_LENGTH + 1,
        "Repeating mode transition did not prune queue"
    );

    assert_eq!(queue.next(), Some(0));
}

#[test]
fn should_transition_to_a_repeating_repeat_mode_properly_with_shuffled_order() {
    let mut queue = generate_queue(
        (0..100).collect(),
        None,
        PlaybackRepeatMode::NoRepeat,
        PlaybackQueueOrder::Shuffle,
    );

    for _ in 0..queue.track_pool.len() - 1 {
        queue.next().unwrap();
    }

    assert_eq!(queue.next(), None);

    assert_eq!(queue.entries.len(), queue.track_pool.len());
    queue.cycle_repeat_mode();
    assert_eq!(
        queue.entries.len(),
        queue.track_pool.len() + PLAYBACK_QUEUE_LENGTH,
        "Repeating mode transition did not truncate queue"
    );

    assert!(
        queue.next().is_some(),
        "Queue didn't generate additional tracks after transition to repeating repeat mode"
    );
}

#[test]
fn should_transition_to_a_non_repeating_repeat_mode_properly_with_sequential_order() {
    let mut queue = generate_queue(
        (0..100).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Sequential,
    );

    for _ in 0..queue.track_pool.len() - 1 {
        queue.next().unwrap();
    }

    assert!(queue.peek_next().is_some());

    assert_eq!(
        queue.entries.len(),
        queue.track_pool.len() + PLAYBACK_QUEUE_LENGTH + 1
    );
    queue.cycle_repeat_mode();
    assert_eq!(
        queue.entries.len(),
        queue.track_pool.len() + PLAYBACK_QUEUE_LENGTH + 1,
        "Transitioning from Repeat to RepeatOne shouldn't affect the queue entries"
    );
    queue.cycle_repeat_mode();
    assert_eq!(
        queue.entries.len(),
        1,
        "Repeating mode transition didn't prune queue"
    );

    assert!(
        queue.next().is_none(),
        "Queue generated tracks when it shouldn't have"
    );

    for _ in 0..queue.track_pool.len() - 1 {
        queue.previous().unwrap();
    }

    assert!(
        queue.previous().is_none(),
        "Queue generated previous tracks when it shouldn't have"
    );
}

#[test]
fn should_transition_to_a_non_repeating_repeat_mode_properly_with_shuffled_order() {
    let mut queue = generate_queue(
        (0..100).collect(),
        None,
        PlaybackRepeatMode::Repeat,
        PlaybackQueueOrder::Shuffle,
    );

    for _ in 0..queue.track_pool.len() - 1 {
        queue.next().unwrap();
    }

    assert!(queue.next().is_some());

    assert_eq!(
        queue.entries.len(),
        queue.track_pool.len() + PLAYBACK_QUEUE_LENGTH + 1
    );
    queue.cycle_repeat_mode();
    assert_eq!(
        queue.entries.len(),
        queue.track_pool.len() + PLAYBACK_QUEUE_LENGTH + 1,
        "Transitioning from Repeat to RepeatOne shouldn't affect the queue entries"
    );
    queue.cycle_repeat_mode();
    assert_eq!(
        queue.entries.len(),
        queue.track_pool.len() + 1,
        "Repeating mode transition didn't truncate queue"
    );

    assert!(
        queue.next().is_none(),
        "Queue generated tracks when it shouldn't have"
    );

    for _ in 0..queue.entries.len() - 1 {
        queue.previous().unwrap();
    }

    assert!(
        queue.previous().is_none(),
        "Queue generated previous tracks when it shouldn't have"
    );
}

#[test]
fn should_not_repeat_user_queued_tracks_when_shuffling_without_repeats() {
    for _ in 0..25 {
        let mut queue = generate_queue(
            (0..100).collect(),
            None,
            PlaybackRepeatMode::NoRepeat,
            PlaybackQueueOrder::Shuffle,
        );

        queue.truncate();
        for index in 1..10 {
            queue.insert_next(index);
        }

        for _ in 0..10 {
            queue.next().unwrap();
        }

        for _ in 0..queue.track_pool.len() - 11 {
            assert!(queue.next().unwrap() >= 10, "Repeated user queued tracks");
        }

        assert_eq!(queue.next(), None);
    }
}

#[test]
fn should_not_repeat_user_queued_tracks_when_shuffling_with_repeats_until_entries_are_past_threshold()
 {
    for _ in 0..25 {
        let mut queue = generate_queue(
            (0..1000).collect(),
            None,
            PlaybackRepeatMode::Repeat,
            PlaybackQueueOrder::Shuffle,
        );

        queue.truncate();
        for index in 1..100 {
            queue.insert_next(index);
        }

        let mut played_tracks = FxHashSet::default();

        let no_repeat_threshold = queue.track_pool.len() / 2;

        for index in 0..queue.track_pool.len() * 2 {
            if index >= no_repeat_threshold {
                let track_id_that_can_repeat = queue.entries[index - no_repeat_threshold];

                played_tracks.remove(&track_id_that_can_repeat.track_id);
            }

            let next_track_id = queue.next().unwrap_or_else(|| {
                panic!("Repeated Shuffled queue ran out of tracks at index {index}")
            });

            assert!(
                !played_tracks.contains(&next_track_id),
                "Repeated track id: {next_track_id}"
            );
            played_tracks.insert(next_track_id);
        }
    }
}
