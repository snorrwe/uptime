CREATE INDEX idx_history_entry_id_created ON status_history(status_entry_id, created DESC);

CREATE INDEX idx_status_history_status_id ON status_history(status_id);
