
-- create user table
CREATE TABLE IF NOT EXISTS users(
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    ws_id BIGINT NOT NULL ,
    -- hashed argon2 password
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL REFERENCES users(id) ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- create chat type: single, group,private_channel, public_channel
CREATE TYPE chat_type AS ENUM('single', 'group', 'private_channel', 'public_channel');

--create chat table
CREATE TABLE IF NOT EXISTS chats(
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) ,
    type chat_type NOT NULL,
    ws_id BIGINT NOT NULL REFERENCES workspaces(id),
    -- user_id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

BEGIN;
INSERT INTO users (fullname, email, ws_id, password_hash) VALUES ('admin', 'super@none.org', 1, '');
-- INSERT INTO users (fullname, email, ws_id, password_hash) VALUES ('Zack Chen', 'zack.j.chen@hkjc.org.hk', 2, '$argon2id$v=19$m=19456,t=2,p=1$CMiNeL3qUUNNygX3ly7ENw$RRTU4A1Qqg4jYqAuB6k3fTpwVB3MA3OissVQsyrND4U');
-- INSERT INTO users (fullname, email, ws_id, password_hash) VALUES ('Goh Zixin', 'zixingoh@hkjc.org.hk', 2, '$argon2id$v=19$m=19456,t=2,p=1$Fql/yCxLjjJyQm05hhtK1Q$qv0Zt+H1TWzw/jrOiJKXclxdCJknFTROOzlXB9DGmWU');

INSERT INTO workspaces (name, owner_id) VALUES ('default', 1);
-- INSERT INTO workspaces (name, owner_id) VALUES ('test', 1);

INSERT INTO chats (name, type, ws_id, members) VALUES ('general', 'public_channel', 1, ARRAY[0]);
-- insert into chats (name, type, ws_id, members) values ('chat-1', 'single', 1, ARRAY [1, 2]);

UPDATE users SET ws_id = 0 WHERE id = 0;
COMMIT;
ALTER TABLE users
  ADD CONSTRAINT users_ws_id_fk FOREIGN KEY (ws_id) REFERENCES workspaces(id);
-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);



-- create message table
CREATE TABLE IF NOT EXISTS messages(
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    files TEXT[] default '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for messages for chat_id and create_at order by desc create_at desc
CREATE INDEX IF NOT EXISTS chat_id_index ON messages(chat_id, created_at DESC);
-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);
