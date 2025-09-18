CREATE TABLE song_suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    song_name TEXT NOT NULL,
    artist TEXT NOT NULL,
    suggested_by_id TEXT NOT NULL,
    suggested_by_name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_song_suggestions_created_at ON song_suggestions(created_at DESC);
CREATE INDEX idx_song_suggestions_user ON song_suggestions(suggested_by_id);
