use sqlx::SqlitePool;

use crate::app::Entry;

pub async fn init_statuses(db: SqlitePool, entries: &[Entry]) -> anyhow::Result<()> {
    for entry in entries {
        sqlx::query!(
            r#"
        INSERT INTO status_entry (name, public_url, internal_url)
        VALUES (?, ?, ?)
        ON CONFLICT DO NOTHING
"#,
            entry.name.as_str(),
            entry.public_url,
            entry.internal_url
        )
        .executre(&db)
        .await?;
    }
    Ok(())
}

pub async fn poll_statuses(db: SqlitePool) {}
