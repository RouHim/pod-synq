-- Favorite episodes
CREATE TABLE IF NOT EXISTS favorite_episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    podcast_url TEXT NOT NULL,
    episode_url TEXT NOT NULL,
    title TEXT,
    podcast_title TEXT,
    description TEXT,
    website TEXT,
    released TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE(user_id, episode_url)
);

CREATE INDEX IF NOT EXISTS idx_favorites_user ON favorite_episodes(user_id);
