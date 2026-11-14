PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS categories (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    parent_id   INTEGER REFERENCES categories(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS videos (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id             INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    platform                TEXT NOT NULL DEFAULT 'bilibili',
    platform_id             TEXT NOT NULL,
    title                   TEXT NOT NULL,
    author                  TEXT,
    cover_url               TEXT,
    duration                INTEGER,
    description             TEXT,
    original_url            TEXT,
    local_video_path        TEXT,
    local_audio_path        TEXT,
    transcript_path         TEXT,
    local_cover_path        TEXT,
    local_subtitle_paths    TEXT NOT NULL DEFAULT '[]',
    aid                     INTEGER,
    cid                     INTEGER,
    description_fetched_at  TEXT,
    is_ai_sorted            INTEGER NOT NULL DEFAULT 0,
    ai_confidence           REAL,
    ai_suggestion           TEXT,
    ai_confirmed            INTEGER NOT NULL DEFAULT 0,
    created_at              TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(platform, platform_id)
);

CREATE INDEX IF NOT EXISTS idx_videos_category ON videos(category_id);
CREATE INDEX IF NOT EXISTS idx_videos_updated ON videos(updated_at DESC);

CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL DEFAULT '#1976D2'
);

CREATE TABLE IF NOT EXISTS video_tags (
    video_id  INTEGER NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
    tag_id    INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (video_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_video_tags_tag ON video_tags(tag_id);

CREATE TABLE IF NOT EXISTS topics (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    color       TEXT NOT NULL DEFAULT '#1976D2',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS video_topics (
    video_id  INTEGER NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
    topic_id  INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    PRIMARY KEY (video_id, topic_id)
);

CREATE TABLE IF NOT EXISTS notebooks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS notebook_videos (
    notebook_id INTEGER NOT NULL REFERENCES notebooks(id) ON DELETE CASCADE,
    video_id    INTEGER NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (notebook_id, video_id)
);

CREATE INDEX IF NOT EXISTS idx_notebook_videos_video ON notebook_videos(video_id);

CREATE TABLE IF NOT EXISTS wbi_cache (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    img_key    TEXT NOT NULL,
    sub_key    TEXT NOT NULL,
    fetched_at INTEGER NOT NULL
);