use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use sqlx::{Acquire, SqlitePool};
use tracing::{debug, error, info};

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
        let internal_url = entry.polling_url.as_ref().map(|x| x.as_str());
        sqlx::query!(
            r#"
INSERT INTO
    status_entry (name, public_url, internal_url)
VALUES
    ($1, $2, $3) ON CONFLICT DO
UPDATE
SET
    public_url = $2,
    internal_url = $3
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
DELETE FROM
    status_history
WHERE
    status_id = $1;

DELETE FROM
    status_entry
WHERE
    id = $1;
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

// TODO: config interval
pub async fn poll_statuses(db: SqlitePool, interval: Duration) -> anyhow::Result<()> {
    loop {
        info!("Polling site statuses");
        if let Err(err) = poll_statuses_once(&db).await {
            error!(?err, "Status poll failed");
        }
        tokio::time::sleep(interval).await;
    }
}

pub async fn poll_statuses_once(db: &SqlitePool) -> anyhow::Result<()> {
    let entries = sqlx::query!(
        r#"
SELECT
    id,
    coalesce(internal_url, public_url) AS url
FROM
    status_entry
"#
    )
    .fetch_all(db)
    .await
    .context("Failed to fetch status entries")?;

    let mut tr = db.begin().await.context("Failed to begin transaction")?;
    let conn = tr
        .acquire()
        .await
        .context("Failed to acquire db connection")?;
    for row in entries {
        let resp = reqwest::get(&row.url).await;
        let status_code = match resp {
            Ok(resp) => resp.status().as_u16() as i64,
            Err(err) => {
                // TODO: record the error in the db
                error!(?err, url = row.url, "Request failed");
                -1
            }
        };

        sqlx::query!(
            r#"
INSERT INTO
    status_history (status_id, status_code)
VALUES
    (?, ?)
            "#,
            row.id,
            status_code
        )
        .execute(&mut *conn)
        .await
        .with_context(|| format!("Failed to insert history entry for {} {}", row.id, row.url))?;
    }
    tr.commit()
        .await
        .context("Failed to commit the transaction")?;
    Ok(())
}
