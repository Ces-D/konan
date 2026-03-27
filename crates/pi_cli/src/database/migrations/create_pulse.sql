CREATE TABLE IF NOT EXISTS pulse (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    start_date INTEGER NOT NULL,
    r_rule TEXT NOT NULL,
    last_run INTEGER NOT NULL,
);

