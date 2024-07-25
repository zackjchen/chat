INSERT INTO workspaces (name, owner_id)
VALUES
('test1', 0),
('test2', 0),
('test3', 0)
;



INSERT INTO users(ws_id, email, fullname,password_hash)
VALUES
 (1, 'zack@email.com', 'Zack', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk'),
 (1, 'bency@email.com', 'lijiajia', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk'),
 (1, 'gaoyin@email.com', 'gaoyin', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk'),
 (1, 'zixin@email.com', 'zixin', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk');




INSERT INTO chats(ws_id, name, type, members)
VALUES
(1, '聊天室1', 'group', '{1,2,3,4}'),
(1, NULL, 'single', '{1,3}'),
(1, '聊天室3', 'public_channel', '{2,3,4}'),
(1, '聊天室4', 'private_channel', '{1,2,4}');
