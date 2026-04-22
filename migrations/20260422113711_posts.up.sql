-- Add up migration script here

CREATE TABLE IF NOT EXISTS posts (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    author_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       VARCHAR(255) NOT NULL,
    body        TEXT        NOT NULL,
    published   BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_posts_author_id  ON posts(author_id);
CREATE INDEX IF NOT EXISTS idx_posts_published  ON posts(published);
