CREATE TABLE game_suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_name TEXT NOT NULL,
    developer TEXT NOT NULL,
    suggested_by_id TEXT NOT NULL,
    suggested_by_name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_game_suggestions_created_at ON game_suggestions(created_at DESC);
CREATE INDEX idx_game_suggestions_user ON game_suggestions(suggested_by_id);
