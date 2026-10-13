ALTER TABLE f_i_virtual_payment_products
    ADD COLUMN allowed_user_ids UUID[] NOT NULL DEFAULT '{}';
COMMENT ON COLUMN f_i_virtual_payment_products.allowed_user_ids IS '允许购买的账号 UUID；空数组表示普通公开商品；V3 测试商品必须指定账号';

ALTER TABLE f_i_virtual_payment_products
    DROP CONSTRAINT virtual_product_key_matches_price,
    ADD CONSTRAINT virtual_product_key_matches_price CHECK (
        product_key ~ '^(membership:V(3|[6-9])|match_id_unlock|reflux_subscription:[^:@]+:[^:@]+)@[1-9][0-9]*$'
        AND split_part(product_key, '@', 2) = price_cents::text
    ),
    ADD CONSTRAINT virtual_v3_product_must_be_restricted CHECK (
        product_key NOT LIKE 'membership:V3@%' OR cardinality(allowed_user_ids) > 0
    ),
    ADD CONSTRAINT virtual_product_allowed_users_not_null CHECK (
        array_position(allowed_user_ids, NULL) IS NULL
    );
