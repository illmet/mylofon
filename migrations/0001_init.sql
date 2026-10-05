CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lookup_hash TEXT NOT NULL UNIQUE CHECK (length(lookup_hash) BETWEEN 1 AND 256),
    password_hash TEXT NOT NULL CHECK (length(password_hash) BETWEEN 1 AND 1024),
    public_id TEXT NOT NULL UNIQUE CHECK (length(public_id) BETWEEN 1 AND 64),
    animal INTEGER NOT NULL CHECK (animal BETWEEN 0 AND 14),
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    author_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    body TEXT NOT NULL CHECK (
        length(body) BETWEEN 1 AND 5000 AND instr(body, char(0)) = 0
    ),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    CHECK (parent_id IS NULL OR parent_id < id)
) STRICT;

-- Both the feed and each reply thread use an indexed cursor, never OFFSET.
CREATE INDEX posts_parent_id_id ON posts(parent_id, id DESC);
CREATE INDEX posts_author_feed ON posts(author_id, id DESC) WHERE parent_id IS NULL;

CREATE TABLE likes (
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (post_id, user_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX likes_user_id ON likes(user_id);
