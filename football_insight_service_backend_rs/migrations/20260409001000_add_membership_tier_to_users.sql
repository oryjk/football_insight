ALTER TABLE f_i_users
    ADD COLUMN IF NOT EXISTS membership_tier VARCHAR(16) NOT NULL DEFAULT 'V1';

UPDATE f_i_users
   SET membership_tier = 'V3'
 WHERE id IN (
     SELECT used_by_user_id
       FROM f_i_invite_codes
      WHERE used_by_user_id IS NOT NULL
 );

ALTER TABLE f_i_users
    DROP CONSTRAINT IF EXISTS chk_f_i_users_membership_tier;

ALTER TABLE f_i_users
    ADD CONSTRAINT chk_f_i_users_membership_tier
    CHECK (membership_tier ~ '^V[0-9]+$');
