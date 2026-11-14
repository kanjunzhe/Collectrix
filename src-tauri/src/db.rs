use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;
use tauri::Manager;

/// videos 表在旧库上需要补齐的列。新库由 001_init.sql 一次建好，这里是兼容路径。
const VIDEO_COMPAT_COLUMNS: &[(&str, &str)] = &[
    ("local_video_path", "TEXT"),
    ("local_audio_path", "TEXT"),
    ("transcript_path", "TEXT"),
    ("local_cover_path", "TEXT"),
    ("local_subtitle_paths", "TEXT NOT NULL DEFAULT '[]'"),
    ("aid", "INTEGER"),
    ("cid", "INTEGER"),
    ("description_fetched_at", "TEXT"),
    ("is_ai_sorted", "INTEGER NOT NULL DEFAULT 0"),
    ("ai_confidence", "REAL"),
    ("ai_suggestion", "TEXT"),
    ("ai_confirmed", "INTEGER NOT NULL DEFAULT 0"),
];

pub fn init_db(app_handle: &tauri::AppHandle) -> Connection {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    std::fs::create_dir_all(&app_dir).ok();
    open_and_migrate(&app_dir.join("video_vault.db"))
}

/// 打开数据库并补齐 schema，每次启动都会执行。
pub fn open_and_migrate(db_path: &Path) -> Connection {
    let conn = Connection::open(db_path).expect("open db failed");

    // 连接级 PRAGMA 只对本次连接生效，必须每次设置：
    // - busy_timeout：和后台任务/播放窗口抢锁时排队而不是立刻报 database is locked
    // - temp_store=MEMORY：避免临时表/排序落到磁盘
    // - cache_size=-8000：8 MB 页缓存（默认 2 MB）
    let _ = conn.busy_timeout(Duration::from_secs(5));
    let _ = conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA cache_size = -8000;",
    );

    // 全部 schema 维护在 001_init.sql；下面的列检查只用于兼容旧数据库。
    conn.execute_batch(include_str!("../migrations/001_init.sql"))
        .expect("migration failed");

    // 兼容升级：表结构只读一次，避免为每一列重复 PRAGMA table_info。
    let video_columns = table_columns(&conn, "videos");
    for (col, ty) in VIDEO_COMPAT_COLUMNS {
        if !video_columns.contains(*col) {
            let _ = conn.execute(&format!("ALTER TABLE videos ADD COLUMN {col} {ty}"), []);
        }
    }
    if !table_columns(&conn, "tags").contains("color") {
        let _ = conn.execute(
            "ALTER TABLE tags ADD COLUMN color TEXT NOT NULL DEFAULT '#1976D2'",
            [],
        );
    }

    ensure_uncategorized(&conn);
    conn
}

fn table_columns(conn: &Connection, table: &str) -> HashSet<String> {
    let mut columns = HashSet::new();
    let Ok(mut stmt) = conn.prepare(&format!("PRAGMA table_info({table})")) else {
        return columns;
    };
    if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) {
        columns.extend(rows.flatten());
    }
    columns
}

/// 确保有一个顶级「未分类」分类，并把历史空分类视频归入其中。
fn ensure_uncategorized(conn: &Connection) {
    let _ = conn.execute(
        "INSERT INTO categories (name, parent_id)
         SELECT '未分类', NULL
         WHERE NOT EXISTS (
           SELECT 1 FROM categories WHERE name='未分类' AND parent_id IS NULL
         )",
        [],
    );
    let uncategorized_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM categories WHERE name='未分类' AND parent_id IS NULL ORDER BY id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();
    if let Some(id) = uncategorized_id {
        let _ = conn.execute(
            "UPDATE videos SET category_id=?1 WHERE category_id IS NULL",
            params![id],
        );
    }
}
