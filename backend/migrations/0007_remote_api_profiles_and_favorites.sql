ALTER TABLE users ADD COLUMN signature TEXT NOT NULL DEFAULT '';
ALTER TABLE users ADD COLUMN avatar BLOB;
ALTER TABLE users ADD COLUMN avatar_content_type TEXT;

CREATE TABLE favorites (
    user_id TEXT NOT NULL,
    article_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (user_id, article_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE
);

CREATE INDEX idx_favorites_article_id ON favorites(article_id);
