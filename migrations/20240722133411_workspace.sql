
-- Add migration script here
CREATE TABLE workspace (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL REFERENCES users(id) ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE users ADD COLUMN ws_id BIGINT REFERENCES workspace(id) ;


BEGIN;
INSERT INTO users(id, fullname, email, password_hash) VALUES (0, 'admin', 'super@none.org', '');
INSERT INTO workspace(id,name, owner_id) VALUES (0,'default', 0);
UPDATE users SET ws_id = 0 WHERE id = 0;
COMMIT;
ALTER TABLE users ALTER COLUMN ws_id SET NOT NULL;
END;
