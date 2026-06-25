CREATE TABLE IF NOT EXISTS configs (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT NOT NULL DEFAULT '',
    locked     INTEGER NOT NULL DEFAULT 0,
    color_idx  INTEGER,
    chip_salt  TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS supplies (
    id        TEXT PRIMARY KEY,
    config_id TEXT NOT NULL REFERENCES configs(id) ON DELETE CASCADE,
    name      TEXT NOT NULL,
    cost      REAL NOT NULL,
    quantity  REAL NOT NULL,
    unit      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS time_costs (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    config_id     TEXT REFERENCES configs(id) ON DELETE SET NULL,
    name          TEXT NOT NULL,
    duration      REAL NOT NULL,
    duration_unit TEXT NOT NULL DEFAULT 'mins',
    rate          REAL NOT NULL,
    color_idx     INTEGER
);

CREATE TABLE IF NOT EXISTS groups (
    id        TEXT PRIMARY KEY,
    user_id   TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name      TEXT NOT NULL,
    x         REAL NOT NULL DEFAULT 20,
    y         REAL NOT NULL DEFAULT 20,
    width     REAL NOT NULL DEFAULT 240,
    height    REAL NOT NULL DEFAULT 180,
    color_idx INTEGER
);

CREATE TABLE IF NOT EXISTS items (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    config_id    TEXT REFERENCES configs(id) ON DELETE SET NULL,
    name         TEXT NOT NULL,
    price        REAL,
    time_cost_id TEXT REFERENCES time_costs(id) ON DELETE SET NULL,
    time_amount  REAL,
    time_unit    TEXT,
    x            REAL,
    y            REAL,
    width        REAL,
    height       REAL,
    group_id     TEXT REFERENCES groups(id) ON DELETE SET NULL,
    rx           REAL,
    ry           REAL
);

CREATE TABLE IF NOT EXISTS item_usages (
    id        TEXT PRIMARY KEY,
    item_id   TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    supply_id TEXT NOT NULL REFERENCES supplies(id) ON DELETE CASCADE,
    amount    REAL NOT NULL,
    unit      TEXT
);
