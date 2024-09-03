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
BEGIN
    IF TG_OP = 'INSERT' THEN
        RAISE NOTICE 'message_added: %', NEW;
        PERFORM pg_notify('message_added', row_to_json(NEW)::text);
    END IF;
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER add_to_message_trigger
AFTER INSERT ON messages
FOR EACH ROW
EXECUTE FUNCTION notify_message_added();
