CREATE TABLE IF NOT EXISTS songs (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  artist TEXT NOT NULL,
  file_path TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  deleted_at TEXT
);

CREATE TRIGGER IF NOT EXISTS songs_updated_at AFTER
UPDATE ON songs FOR EACH ROW BEGIN
UPDATE songs
SET
  updated_at = datetime('now')
WHERE
  id = NEW.id;

END;
