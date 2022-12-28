-- Your SQL goes here
CREATE TABLE pending_posts
(
    id        TEXT UNIQUE PRIMARY KEY              NOT NULL,
    author_id TEXT                                 NOT NULL,
    excerpt   TEXT                                 NOT NULL,
    citation  TEXT                                 NOT NULL,
    creation  DATETIME DEFAULT (current_timestamp) NOT NULL
);

CREATE TABLE islamism
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES pending_posts (id)
);

CREATE TABLE modernity
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES pending_posts (id)
);

CREATE TABLE secularism
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES pending_posts (id)
);

CREATE TABLE feminism
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES pending_posts (id)
);
