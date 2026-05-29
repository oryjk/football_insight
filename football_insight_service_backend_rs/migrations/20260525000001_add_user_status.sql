ALTER TABLE f_i_users
    ADD COLUMN IF NOT EXISTS status VARCHAR(16) NOT NULL DEFAULT 'active';

UPDATE f_i_users
   SET status = 'active'
 WHERE status IS NULL;

ALTER TABLE f_i_users
    DROP CONSTRAINT IF EXISTS chk_f_i_users_status;

ALTER TABLE f_i_users
    ADD CONSTRAINT chk_f_i_users_status
    CHECK (status IN ('active', 'disabled'));

CREATE INDEX IF NOT EXISTS idx_f_i_users_status_created_at
    ON f_i_users (status, created_at DESC);
