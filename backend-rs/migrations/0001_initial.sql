CREATE TABLE IF NOT EXISTS users (
    id            TEXT PRIMARY KEY,
    email         TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_state (
    user_id    TEXT PRIMARY KEY,
    state_json TEXT NOT NULL DEFAULT '{"configs":[],"items":[],"groups":[],"timeCosts":[]}',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
