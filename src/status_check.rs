use std::collections::HashMap;

use anyhow::Context;
use sqlx::{Acquire, SqlitePool};
use tracing::debug;

use crate::app::Entry;

pub async fn init_statuses(db: &SqlitePool, entries: &[Entry]) -> anyhow::Result<()> {
    let mut tr = db.begin().await.context("Failed to start transaction")?;
    let conn = tr
        .acquire()
        .await
        .context("Failed to acquire db connection")?;

    let existing_entries = sqlx::query!(r#"SELECT id, name FROM status_entry"#)
        .fetch_all(&mut *conn)
        .await
        .context("Failed to fetch existing entries")?;

    let mut existing_entries = existing_entries
        .into_iter()
        .map(|r| (r.name, r.id))
        .collect::<HashMap<String, i64>>();

    for entry in entries {
        let name = entry.name.as_str();
        existing_entries.remove(name);
        let public_url = entry.public_url.as_str();
        let internal_url = entry.internal_url.as_ref().map(|x| x.as_str());
        sqlx::query!(
            r#"
        INSERT INTO status_entry (name, public_url, internal_url)
        VALUES ($1, $2, $3)
        ON CONFLICT DO UPDATE
        set public_url=$2, internal_url=$3
"#,
            name,
            public_url,
            internal_url
        )
        .execute(&mut *conn)
        .await
        .with_context(|| format!("Failed to insert entry {}", entry.name))?;
    }

    for (name, id) in existing_entries {
        debug!(name, id, "Removing missing entry");
        sqlx::query!(
            r#"
        DELETE FROM status_history WHERE status_id=$1;
        DELETE FROM status_entry WHERE id=$1;
"#,
            id,
            id
        )
        .execute(&mut *conn)
        .await
        .with_context(|| format!("Failed to delete missing entry {}", name))?;
    }

    tr.commit().await.context("Failed to commit transaction")?;

    Ok(())
}

pub async fn poll_statuses(db: SqlitePool) {}
