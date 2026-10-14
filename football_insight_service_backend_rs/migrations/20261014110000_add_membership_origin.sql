ALTER TABLE f_i_users ADD COLUMN membership_origin TEXT NOT NULL DEFAULT 'unknown'
    CHECK (membership_origin IN ('purchase', 'referral', 'unknown'));
ALTER TABLE f_i_virtual_payment_orders
    ADD COLUMN grant_before_origin TEXT NOT NULL DEFAULT 'unknown',
    ADD COLUMN grant_after_origin TEXT NOT NULL DEFAULT 'unknown';

UPDATE f_i_virtual_payment_orders SET grant_after_origin = 'purchase'
WHERE grant_before IS NOT NULL AND grant_after IS NOT NULL
  AND grant_before IS DISTINCT FROM grant_after;
UPDATE f_i_virtual_payment_orders v SET grant_before_origin = 'purchase'
WHERE EXISTS (
    SELECT 1 FROM f_i_virtual_payment_orders previous
    JOIN f_i_payment_orders p ON p.order_no = previous.order_no
    JOIN f_i_payment_orders current_order ON current_order.order_no = v.order_no
    WHERE p.user_id = current_order.user_id AND p.status = 'paid'
      AND previous.order_no <> v.order_no AND previous.grant_after_origin = 'purchase'
      AND previous.grant_after = v.grant_before AND p.paid_at <= current_order.paid_at
);

-- A payment proves the current grant only when its recorded state matches exactly.
-- Referral counts alone do not prove which action granted a historical membership.
UPDATE f_i_users u SET membership_origin = 'purchase'
WHERE EXISTS (
    SELECT 1 FROM f_i_payment_orders p
    LEFT JOIN f_i_virtual_payment_orders v ON v.order_no = p.order_no
    WHERE p.user_id = u.id AND p.status = 'paid' AND p.paid_at IS NOT NULL
      AND p.product_type = 'membership:' || u.membership_tier
      AND (
        v.grant_after = jsonb_build_object('tier', u.membership_tier, 'expires_at', u.membership_expires_at)
        OR (v.order_no IS NULL AND u.membership_expires_at = p.paid_at + INTERVAL '1 year')
      )
);

CREATE FUNCTION f_i_track_membership_origin() RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE origin TEXT;
BEGIN
    IF NEW.membership_tier IS DISTINCT FROM OLD.membership_tier
       OR NEW.membership_expires_at IS DISTINCT FROM OLD.membership_expires_at THEN
        origin := current_setting('football.membership_origin', true);
        NEW.membership_origin := CASE WHEN origin IN ('purchase', 'referral') THEN origin ELSE 'unknown' END;
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER f_i_users_membership_origin
BEFORE UPDATE ON f_i_users FOR EACH ROW EXECUTE FUNCTION f_i_track_membership_origin();
