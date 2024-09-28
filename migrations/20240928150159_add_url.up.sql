CREATE TABLE status_entry_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name VARCHAR NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP,
    public_url VARCHAR NOT NULL,
    internal_url VARCHAR NULL
);

INSERT INTO status_entry_new
SELECT *, 'https://example.com' AS public_url, NULL AS internal_url
FROM status_entry;

DROP TABLE status_entry;

ALTER TABLE status_entry_new RENAME TO status_entry;
