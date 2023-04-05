CREATE TABLE attributes (
    path TEXT NOT NULL,
    attr_key TEXT NOT NULL,
    attr_value TEXT NOT NULL,
    PRIMARY KEY (path, attr_key, attr_value)
);
