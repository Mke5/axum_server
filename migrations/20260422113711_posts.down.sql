-- Add down migration script here
DROP INDEX IF EXISTS idx_posts_author_id;
DROP INDEX IF EXISTS idx_posts_published;

DROP TABLE IF EXISTS posts;
DROP EXTENSION IF EXISTS "uuid-ossp";
