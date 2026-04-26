ALTER TABLE f_i_invite_codes
    ADD COLUMN IF NOT EXISTS issued_for_wechat_open_id VARCHAR(128) NULL;

ALTER TABLE f_i_users
    ADD COLUMN IF NOT EXISTS official_wechat_open_id VARCHAR(128) NULL;

UPDATE f_i_invite_codes AS invite_codes
   SET issued_for_wechat_open_id = followers.open_id
  FROM f_i_wechat_followers AS followers
 WHERE followers.latest_invite_code_id = invite_codes.id
   AND invite_codes.issued_for_wechat_open_id IS NULL;

UPDATE f_i_users AS users
   SET official_wechat_open_id = invite_codes.issued_for_wechat_open_id
  FROM f_i_invite_codes AS invite_codes
 WHERE invite_codes.used_by_user_id = users.id
   AND invite_codes.issued_for_wechat_open_id IS NOT NULL
   AND users.official_wechat_open_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_f_i_invite_codes_issued_for_wechat_open_id
    ON f_i_invite_codes (issued_for_wechat_open_id);

CREATE INDEX IF NOT EXISTS idx_f_i_users_official_wechat_open_id
    ON f_i_users (official_wechat_open_id);
