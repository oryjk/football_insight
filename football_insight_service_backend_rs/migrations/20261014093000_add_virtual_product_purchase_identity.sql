-- Stop old payment writers during migration+deployment: older binaries cannot record catalog UUIDs.
-- No seed data or foreign key is added; deleting a SKU must never delete historical orders.
ALTER TABLE f_i_virtual_payment_products ADD COLUMN id UUID NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE f_i_virtual_payment_products ADD CONSTRAINT virtual_product_catalog_identity_unique UNIQUE(id);
COMMENT ON COLUMN f_i_virtual_payment_products.id IS 'Immutable catalog identity retained across price/key/WeChat ID edits';
ALTER TABLE f_i_payment_orders ADD COLUMN catalog_product_id UUID;
COMMENT ON COLUMN f_i_payment_orders.catalog_product_id IS 'Catalog identity at checkout; no FK so deleting/recreating a SKU cannot steal or remove orders';

CREATE FUNCTION f_i_virtual_product_keep_identity() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.id IS DISTINCT FROM OLD.id THEN
        RAISE EXCEPTION 'product catalog identity is immutable' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER virtual_product_identity_immutable BEFORE UPDATE OF id ON f_i_virtual_payment_products
FOR EACH ROW EXECUTE FUNCTION f_i_virtual_product_keep_identity();

-- Only exact typed membership/amount matches are eligible. Generic legacy 'membership' is intentionally unassigned.
-- Existing virtual metadata additionally proves the WeChat ID and captured amount.
-- Explicit cardinality guards ambiguous/corrupt legacy catalogs rather than picking an arbitrary match.
WITH candidates AS (
    SELECT p.order_no,c.id
    FROM f_i_payment_orders p
    JOIN f_i_virtual_payment_products c ON split_part(c.product_key,'@',1)=p.product_type AND c.price_cents=p.amount_cents
    LEFT JOIN f_i_virtual_payment_orders v ON v.order_no=p.order_no
    WHERE p.catalog_product_id IS NULL AND p.product_type ~ '^membership:V(3|[6-9])$'
      AND (v.order_no IS NULL OR (v.product_id=c.product_id AND v.amount_cents=p.amount_cents))
), unique_matches AS (
    SELECT order_no,(array_agg(id))[1] AS id FROM candidates GROUP BY order_no HAVING count(*)=1
)
UPDATE f_i_payment_orders p SET catalog_product_id=u.id FROM unique_matches u WHERE p.order_no=u.order_no AND p.catalog_product_id IS NULL;

CREATE INDEX idx_f_i_payment_orders_catalog_purchases ON f_i_payment_orders(catalog_product_id,paid_at DESC,order_no)
WHERE catalog_product_id IS NOT NULL AND paid_at IS NOT NULL AND status IN ('paid','refunded');
