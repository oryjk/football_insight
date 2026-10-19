-- Keep existing like rows and counts compatible with older backend versions.
CREATE TABLE f_i_lineup_dislikes (
    lineup_id UUID NOT NULL REFERENCES f_i_lineups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (lineup_id, user_id)
);
CREATE INDEX idx_f_i_lineup_dislikes_user_id ON f_i_lineup_dislikes(user_id);

-- Older clients/backends still insert into f_i_lineup_likes. Enforce exclusivity
-- for both tables even while old and new application versions coexist.
CREATE FUNCTION f_i_lineup_exclusive_reaction() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM id FROM f_i_lineups WHERE id = NEW.lineup_id FOR UPDATE;
    IF TG_TABLE_NAME = 'f_i_lineup_likes' THEN
        DELETE FROM f_i_lineup_dislikes WHERE lineup_id = NEW.lineup_id AND user_id = NEW.user_id;
    ELSE
        DELETE FROM f_i_lineup_likes WHERE lineup_id = NEW.lineup_id AND user_id = NEW.user_id;
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER f_i_lineup_like_exclusive BEFORE INSERT OR UPDATE ON f_i_lineup_likes
    FOR EACH ROW EXECUTE FUNCTION f_i_lineup_exclusive_reaction();
CREATE TRIGGER f_i_lineup_dislike_exclusive BEFORE INSERT OR UPDATE ON f_i_lineup_dislikes
    FOR EACH ROW EXECUTE FUNCTION f_i_lineup_exclusive_reaction();
