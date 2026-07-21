-- Rebuilding from scratch the table as we are in a prerelease stage of the project.
DROP TRIGGER IF EXISTS track_updated_at;

DROP INDEX IF EXISTS idx_track_title;

DROP INDEX IF EXISTS idx_track_artist;

DROP INDEX IF EXISTS idx_track_album;

DROP INDEX IF EXISTS idx_track_duration_secs;

DROP INDEX IF EXISTS idx_track_play_count;

DROP INDEX IF EXISTS idx_track_last_played;

DROP INDEX IF EXISTS idx_track_created_at;

DROP INDEX IF EXISTS idx_track_updated_at;

DROP INDEX IF EXISTS idx_track_file_path;

DROP TABLE IF EXISTS track;

CREATE TABLE
    track (
        id INTEGER PRIMARY KEY NOT NULL,
        file_path TEXT NOT NULL,
        file_size_bytes INTEGER NOT NULL CHECK (file_size_bytes >= 0),
        file_format TEXT NOT NULL,
        codec TEXT NOT NULL,
        frames INTEGER NOT NULL CHECK (frames >= 0),
        sample_rate INTEGER NOT NULL CHECK (sample_rate > 0),
        channels INTEGER NOT NULL CHECK (channels > 0),
        bit_depth INTEGER CHECK (bit_depth >= 0),
        bitrate_kbps INTEGER CHECK (bitrate_kbps >= 0),
        title TEXT,
        artist TEXT,
        album TEXT,
        album_artist TEXT,
        track_number INTEGER CHECK (track_number >= 0),
        track_total INTEGER CHECK (track_total >= 0),
        disc_number INTEGER CHECK (disc_number >= 0),
        disc_total INTEGER CHECK (disc_total >= 0),
        year INTEGER,
        genre TEXT,
        replaygain_track_gain_db REAL,
        replaygain_track_peak REAL,
        replaygain_album_gain_db REAL,
        replaygain_album_peak REAL,
        play_count INTEGER NOT NULL DEFAULT 0 CHECK (play_count >= 0),
        skip_count INTEGER NOT NULL DEFAULT 0 CHECK (skip_count >= 0),
        volume_adjustment_db REAL NOT NULL DEFAULT 0.0,
        last_played INTEGER CHECK (last_played >= 0),
        created_at INTEGER NOT NULL DEFAULT (unixepoch ()) CHECK (created_at >= 0),
        updated_at INTEGER NOT NULL DEFAULT (unixepoch ()) CHECK (updated_at >= 0),
        deleted_at INTEGER CHECK (deleted_at >= 0)
    );

CREATE TRIGGER track_updated_at AFTER
UPDATE ON track FOR EACH ROW BEGIN
UPDATE track
SET
    updated_at = unixepoch ()
WHERE
    id = NEW.id;

END;

CREATE INDEX idx_track_title ON track (title)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_track_artist ON track (artist)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_track_album ON track (album)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_track_frames ON track (frames)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_track_play_count ON track (play_count)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_track_last_played ON track (last_played)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_track_created_at ON track (created_at)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_track_updated_at ON track (updated_at)
WHERE
    deleted_at IS NULL;

CREATE UNIQUE INDEX idx_track_file_path ON track (file_path);
