DROP TRIGGER IF EXISTS songs_updated_at;

DROP TABLE IF EXISTS songs;

CREATE TABLE IF NOT EXISTS tracks (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT,
  artist TEXT,
  album TEXT,
  album_artist TEXT,
  track_number INTEGER,
  disc_number INTEGER,
  year INTEGER,
  genre TEXT,
  duration_secs REAL,
  bitrate INTEGER,
  sample_rate INTEGER,
  channels INTEGER,
  file_path TEXT NOT NULL,
  file_size INTEGER,
  format TEXT,
  play_count INTEGER NOT NULL DEFAULT 0,
  skip_count INTEGER NOT NULL DEFAULT 0,
  volume_adjustment REAL NOT NULL DEFAULT 0.0,
  last_played INTEGER,
  rating INTEGER,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
  deleted_at INTEGER
);

CREATE TRIGGER IF NOT EXISTS tracks_updated_at AFTER
UPDATE ON tracks FOR EACH ROW BEGIN
UPDATE tracks
SET
  updated_at = unixepoch()
WHERE
  id = NEW.id;

END;

CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks (title)
WHERE
  deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks (artist)
WHERE
  deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks (album)
WHERE
  deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tracks_duration_secs ON tracks (duration_secs)
WHERE
  deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tracks_play_count ON tracks (play_count)
WHERE
  deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tracks_last_played ON tracks (last_played)
WHERE
  deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tracks_created_at ON tracks (created_at)
WHERE
  deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tracks_updated_at ON tracks (updated_at)
WHERE
  deleted_at IS NULL;

PRAGMA user_version = 2;
