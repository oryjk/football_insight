ALTER TABLE f_i_team_insights
    ADD COLUMN IF NOT EXISTS assists_for_by_player JSONB NOT NULL DEFAULT '[]'::jsonb;
