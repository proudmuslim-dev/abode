-- Your SQL goes here
CREATE TABLE users
(
    id       TEXT UNIQUE PRIMARY KEY NOT NULL,
    username TEXT UNIQUE             NOT NULL,
    password TEXT                    NOT NULL,
    admin    BOOLEAN                 NOT NULL
);

CREATE TABLE posts
(
    id        TEXT UNIQUE PRIMARY KEY NOT NULL,
    author_id TEXT                    NOT NULL,
    excerpt   TEXT                    NOT NULL,
    citation  TEXT                    NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users (id)
);

/*Will create more sections soon 🚎*/
CREATE TABLE islamism
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts (id)
);

CREATE TABLE modernity
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts (id)
);

CREATE TABLE secularism
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts (id)
);

CREATE TABLE feminism
(
    post_id TEXT UNIQUE PRIMARY KEY NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts (id)
);
