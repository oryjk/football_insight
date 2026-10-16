-- P1 only: preserve every catalog identity, existing purchase rule, and legacy order.
-- No SKU seed, catalog_source switch, or grant snapshot backfill is performed here.
-- Existing Admin/legacy INSERT writers omit kind; they create membership products.
-- The explicit prefix backfill below still determines each existing row's kind.
ALTER TABLE f_i_virtual_payment_products
    ADD COLUMN kind TEXT DEFAULT 'membership',
    ADD COLUMN grant_spec JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN pricing_rule JSONB NOT NULL DEFAULT '{"type":"fixed"}'::jsonb,
    ADD COLUMN eligibility JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN requires_capability TEXT,
    ADD COLUMN title TEXT,
    ADD COLUMN subtitle TEXT,
    ADD COLUMN description TEXT,
    ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN group_key TEXT;

UPDATE f_i_virtual_payment_products
SET kind = split_part(split_part(product_key, '@', 1), ':', 1),
    grant_spec = CASE
        WHEN product_key LIKE 'membership:%' THEN
            jsonb_build_object(
                'tier', split_part(split_part(product_key, '@', 1), ':', 2),
                'expiry', jsonb_build_object('rule', 'months', 'months', membership_duration_months)
            )
        ELSE '{}'::jsonb
    END;

ALTER TABLE f_i_virtual_payment_products
    ALTER COLUMN kind SET NOT NULL,
    DROP CONSTRAINT virtual_product_key_matches_price,
    ADD CONSTRAINT virtual_product_key_matches_price CHECK (
        product_key ~ '^(membership:V[3-9]|match_id_unlock|reflux_subscription:[^:@]+:[^:@]+)@[1-9][0-9]*$'
        AND split_part(product_key, '@', 2) = price_cents::text
    );
-- virtual_v3_product_must_be_restricted and allowed-user constraints remain intact.

ALTER TABLE f_i_virtual_payment_orders ADD COLUMN grant_snapshot JSONB;
COMMENT ON COLUMN f_i_virtual_payment_orders.grant_snapshot IS
    'Immutable checkout grant specification; NULL preserves historical settlement behavior';
