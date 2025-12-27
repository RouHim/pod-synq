-- Migration 006: Add podcasts metadata table
-- This migration adds a podcasts table to store podcast metadata and auto-track subscriber counts

-- Podcasts metadata table (minimal)
CREATE TABLE IF NOT EXISTS podcasts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT UNIQUE NOT NULL,
    title TEXT,
    description TEXT,
    website TEXT,
    logo_url TEXT,
    subscriber_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_podcasts_url ON podcasts(url);
CREATE INDEX IF NOT EXISTS idx_podcasts_subscriber_count ON podcasts(subscriber_count DESC);

-- Trigger to auto-track subscriber counts from subscriptions
CREATE TRIGGER IF NOT EXISTS update_subscriber_count
AFTER INSERT ON subscriptions
BEGIN
    INSERT INTO podcasts (url, subscriber_count, updated_at)
    VALUES (NEW.podcast_url, 1, strftime('%s', 'now'))
    ON CONFLICT(url) DO UPDATE SET
        subscriber_count = subscriber_count + 1,
        updated_at = strftime('%s', 'now');
END;

-- Trigger to decrease subscriber count when subscription removed
CREATE TRIGGER IF NOT EXISTS decrease_subscriber_count
AFTER UPDATE OF removed_at ON subscriptions
WHEN NEW.removed_at IS NOT NULL AND OLD.removed_at IS NULL
BEGIN
    UPDATE podcasts
    SET subscriber_count = MAX(0, subscriber_count - 1),
        updated_at = strftime('%s', 'now')
    WHERE url = NEW.podcast_url;
END;

-- Backfill existing subscriptions into podcasts table
INSERT INTO podcasts (url, subscriber_count, created_at, updated_at)
SELECT
    podcast_url,
    COUNT(*) as sub_count,
    MIN(added_at) as created_at,
    strftime('%s', 'now') as updated_at
FROM subscriptions
WHERE removed_at IS NULL
GROUP BY podcast_url
ON CONFLICT(url) DO NOTHING;
