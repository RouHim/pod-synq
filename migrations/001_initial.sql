-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Devices table (auto-created on first use)
CREATE TABLE IF NOT EXISTS devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    caption TEXT,
    type TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE(user_id, device_id)
);

-- Subscriptions table
CREATE TABLE IF NOT EXISTS subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    podcast_url TEXT NOT NULL,
    added_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    removed_at INTEGER NULL,
    UNIQUE(user_id, device_id, podcast_url)
);

-- Episode Actions table
CREATE TABLE IF NOT EXISTS episode_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    podcast_url TEXT NOT NULL,
    episode_url TEXT NOT NULL,
    action TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    started INTEGER,
    position INTEGER,
    total INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Admin Sessions table (for web UI)
CREATE TABLE IF NOT EXISTS admin_sessions (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_subscriptions_user_device ON subscriptions(user_id, device_id);
CREATE INDEX IF NOT EXISTS idx_episode_actions_user ON episode_actions(user_id);
CREATE INDEX IF NOT EXISTS idx_episode_actions_podcast ON episode_actions(podcast_url);
CREATE INDEX IF NOT EXISTS idx_episode_actions_timestamp ON episode_actions(timestamp);
CREATE INDEX IF NOT EXISTS idx_admin_sessions_expires ON admin_sessions(expires_at);
