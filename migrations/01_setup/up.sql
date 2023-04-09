CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL
);

CREATE TABLE tags (
    file_id INTEGER,
    tag TEXT NOT NULL,
    PRIMARY KEY (file_id, tag),
    FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE TABLE attributes (
    file_id INTEGER,
    attr_key TEXT NOT NULL,
    attr_value TEXT NOT NULL,
    PRIMARY KEY (file_id, attr_key, attr_value),
    FOREIGN KEY (file_id) REFERENCES files(id)
);
