PRAGMA foreign_keys = ON;

CREATE TABLE status_entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name VARCHAR NON NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    status_id INTEGER NOT NULL,
    -- -1 for network failure
    status_code INTEGER NOT NULL,
    created DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (status_id) REFERENCES status_entry(id)
);
