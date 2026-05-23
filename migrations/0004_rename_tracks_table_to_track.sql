DROP TRIGGER IF EXISTS tracks_updated_at;

DROP INDEX IF EXISTS idx_tracks_title;
DROP INDEX IF EXISTS idx_tracks_artist;
DROP INDEX IF EXISTS idx_tracks_album;
DROP INDEX IF EXISTS idx_tracks_duration_secs;
DROP INDEX IF EXISTS idx_tracks_play_count;
DROP INDEX IF EXISTS idx_tracks_last_played;
DROP INDEX IF EXISTS idx_tracks_created_at;
DROP INDEX IF EXISTS idx_tracks_updated_at;
DROP INDEX IF EXISTS idx_tracks_file_path;

ALTER TABLE tracks RENAME TO track;

CREATE TRIGGER IF NOT EXISTS track_updated_at AFTER UPDATE ON track
FOR EACH ROW BEGIN
  UPDATE track SET updated_at = unixepoch() WHERE id = NEW.id;
END;

CREATE INDEX IF NOT EXISTS idx_track_title         ON track (title)         WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_track_artist        ON track (artist)        WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_track_album         ON track (album)         WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_track_duration_secs ON track (duration_secs) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_track_play_count    ON track (play_count)    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_track_last_played   ON track (last_played)   WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_track_created_at    ON track (created_at)    WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_track_updated_at    ON track (updated_at)    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_track_file_path ON track (file_path);
