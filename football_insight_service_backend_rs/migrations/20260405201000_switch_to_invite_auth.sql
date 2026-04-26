ALTER TABLE f_i_users
    ALTER COLUMN wx_open_id DROP NOT NULL;

ALTER TABLE f_i_users
    ADD COLUMN IF NOT EXISTS phone_number VARCHAR(32),
    ADD COLUMN IF NOT EXISTS password_hash TEXT;

UPDATE f_i_users
   SET phone_number = COALESCE(phone_number, wx_open_id)
 WHERE phone_number IS NULL
   AND wx_open_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_users_phone_number
    ON f_i_users (phone_number)
    WHERE phone_number IS NOT NULL;

CREATE TABLE IF NOT EXISTS f_i_invite_codes (
    id UUID PRIMARY KEY,
    code VARCHAR(64) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    used_by_user_id UUID NULL REFERENCES f_i_users(id) ON DELETE SET NULL,
    used_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
