CREATE TABLE users_new (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    token TEXT,
    identity TEXT NOT NULL DEFAULT 'user'
        CHECK (identity IN ('admin', 'user', 'visitor'))
);

INSERT INTO users_new (id, username, password, token, identity)
SELECT id, username, password, token, identity
FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;

CREATE TABLE comments_new (
    comment_id TEXT PRIMARY KEY NOT NULL,
    article_id TEXT NOT NULL,
    user TEXT,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    parent_id TEXT,
    like_count INTEGER NOT NULL DEFAULT 0 CHECK (like_count >= 0),
    UNIQUE (comment_id, article_id),
    FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id, article_id)
        REFERENCES comments_new(comment_id, article_id) ON DELETE CASCADE
);

INSERT INTO comments_new (
    comment_id,
    article_id,
    user,
    content,
    created_at,
    parent_id,
    like_count
)
SELECT
    comment_id,
    article_id,
    user,
    COALESCE(content, ''),
    COALESCE(created_at, datetime('now')),
    parent_id,
    COALESCE(like_count, 0)
FROM comments;

CREATE TABLE comment_likes_new (
    comment_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    created_at TEXT,
    article_id TEXT NOT NULL,
    PRIMARY KEY (comment_id, user_id),
    FOREIGN KEY (comment_id, article_id)
        REFERENCES comments_new(comment_id, article_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE
);

INSERT INTO comment_likes_new (comment_id, user_id, created_at, article_id)
SELECT likes.comment_id, likes.user_id, likes.created_at, comments.article_id
FROM comment_likes AS likes
LEFT JOIN comments ON comments.comment_id = likes.comment_id;

UPDATE comments_new
SET like_count = (
    SELECT COUNT(*)
    FROM comment_likes_new
    WHERE comment_likes_new.comment_id = comments_new.comment_id
);

DROP TABLE comment_likes;
DROP TABLE comments;
ALTER TABLE comments_new RENAME TO comments;
ALTER TABLE comment_likes_new RENAME TO comment_likes;

CREATE INDEX idx_comments_article_id ON comments(article_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_id);
CREATE INDEX idx_comment_likes_user_id ON comment_likes(user_id);
CREATE INDEX idx_comment_likes_article_id ON comment_likes(article_id);
