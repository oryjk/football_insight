DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
          FROM information_schema.columns
         WHERE table_name = 'f_i_users'
           AND column_name = 'account_identifier'
    ) THEN
        ALTER TABLE f_i_users
            ADD COLUMN account_identifier VARCHAR(64);
    END IF;
END $$;

UPDATE f_i_users
   SET account_identifier = COALESCE(
       account_identifier,
       phone_number,
       CONCAT('wx_', wx_open_id)
   )
 WHERE account_identifier IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_f_i_users_account_identifier
    ON f_i_users (account_identifier)
    WHERE account_identifier IS NOT NULL;
