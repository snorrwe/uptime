use sqlx::SqlitePool;

use crate::app::Entry;

pub async fn init_statuses(db: &SqlitePool, entries: &[Entry]) -> anyhow::Result<()> {
    for entry in entries {
        let name = entry.name.as_str();
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
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn poll_statuses(db: SqlitePool) {}
