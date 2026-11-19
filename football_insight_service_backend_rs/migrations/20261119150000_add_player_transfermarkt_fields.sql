-- Add Transfermarkt player metadata (market value, birth date, contract expiry)
-- to squad players and director candidates.

ALTER TABLE f_i_squad_players
    ADD COLUMN IF NOT EXISTS market_value_eur INTEGER,
    ADD COLUMN IF NOT EXISTS birth_date DATE,
    ADD COLUMN IF NOT EXISTS contract_until DATE,
    ADD COLUMN IF NOT EXISTS transfermarkt_synced_at TIMESTAMPTZ;

ALTER TABLE f_i_director_candidates
    ADD COLUMN IF NOT EXISTS market_value_eur INTEGER,
    ADD COLUMN IF NOT EXISTS birth_date DATE,
    ADD COLUMN IF NOT EXISTS contract_until DATE,
    ADD COLUMN IF NOT EXISTS transfermarkt_synced_at TIMESTAMPTZ;
