ALTER TABLE f_i_matches
    ADD COLUMN IF NOT EXISTS technical_stats JSONB;
