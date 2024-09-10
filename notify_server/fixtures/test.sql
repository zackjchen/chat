INSERT INTO workspaces (name, owner_id)
VALUES
('test1', 1),
('test2', 1),
('test3', 1)
;



INSERT INTO users(ws_id, email, fullname,password_hash)
VALUES
 (2, 'zack@email.com', 'Zack', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk'),
 (2, 'bency@email.com', 'lijiajia', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk'),
 (2, 'gaoyin@email.com', 'gaoyin', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk'),
 (2, 'zixin@email.com', 'zixin', '$argon2id$v=19$m=19456,t=2,p=1$gx8v9D5+5jlThZcuBfmc9w$sGMUKp8uiVQnjdbjaCTbjJDn82C4MpMwo3BboAb7huk');




INSERT INTO chats(ws_id, name, type, members)
VALUES
(2, '聊天室1', 'group', '{2,3,4,5}'), -- id = 1
(2, NULL, 'single', '{2,4}'), -- id = 2
(2, '聊天室3', 'public_channel', '{3,4,5}'), -- id = 3
(2, '聊天室4', 'private_channel', '{2,3,5}'); -- id = 4



INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 2, 'hello1', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 3, 'hello2', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 4, 'hello3', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 5, 'hello4', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 2, 'hello5', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 4, 'hello6', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 3, 'hello7', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 4, 'hello8', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 5, 'hello9', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 2, 'hello10', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 3, 'hello11', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 5, 'hello12', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 2, 'word1', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 3, 'word2', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 4, 'word3', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 5, 'word4', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 2, 'word5', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 4, 'word6', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 3, 'word7', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 4, 'word8', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 5, 'word9', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 2, 'word10', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 3, 'word11', '{}');
INSERT INTO messages(chat_id, sender_id, content, files) VALUES (2, 5, 'word12', '{}');
