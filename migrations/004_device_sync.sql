-- Device synchronization groups
CREATE TABLE IF NOT EXISTS device_sync_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS device_sync_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_group_id INTEGER NOT NULL REFERENCES device_sync_groups(id) ON DELETE CASCADE,
    device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    UNIQUE(device_id)
);

CREATE INDEX IF NOT EXISTS idx_device_sync_groups_user ON device_sync_groups(user_id);
CREATE INDEX IF NOT EXISTS idx_device_sync_members_group ON device_sync_members(sync_group_id);
CREATE INDEX IF NOT EXISTS idx_device_sync_members_device ON device_sync_members(device_id);
