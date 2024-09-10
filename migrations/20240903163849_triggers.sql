-- Add migration script here


-- if user us added to chat, notify with chat data
CREATE OR REPLACE FUNCTION notify_chat_updated()
RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'chat_added: %', NEW;
    PERFORM pg_notify('chat_updated', json_build_object(
        'op', TG_OP,
        'old', OLD,
        'new', NEW
    )::text);
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;


CREATE TRIGGER chat_updated_trigger
AFTER INSERT OR UPDATE OR DELETE ON chats
FOR EACH ROW
EXECUTE FUNCTION notify_chat_updated();



-- if message is added to chat, notify with message data
CREATE OR REPLACE FUNCTION notify_message_added()
RETURNS TRIGGER
AS $$
DECLARE
    users BIGINT[];
BEGIN
    IF TG_OP = 'INSERT' THEN
        RAISE NOTICE 'message_added: %', NEW;
        SELECT members INTO users FROM chats where id=NEW.chat_id;
        PERFORM pg_notify('message_added', json_build_object('message', NEW, 'members', users)::text);
    END IF;
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER add_to_message_trigger
AFTER INSERT ON messages
FOR EACH ROW
EXECUTE FUNCTION notify_message_added();
