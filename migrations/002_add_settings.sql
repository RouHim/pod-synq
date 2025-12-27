-- Settings table for user/device/podcast/episode settings
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scope TEXT NOT NULL,
    podcast_url TEXT,
    device_id INTEGER REFERENCES devices(id) ON DELETE CASCADE,
    episode_url TEXT,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE(user_id, scope, podcast_url, device_id, episode_url, key)
);

-- Indexes for settings
CREATE INDEX IF NOT EXISTS idx_settings_user_scope ON settings(user_id, scope);
CREATE INDEX IF NOT EXISTS idx_settings_podcast ON settings(podcast_url);
CREATE INDEX IF NOT EXISTS idx_settings_device ON settings(device_id);
