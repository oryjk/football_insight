-- 商品映射已经迁入 f_i_virtual_payment_products；下列仅为添加商品的 SQL 示例。
-- 先核对业务当前价格/队伍 code，并在微信后台发布同 ID 同价格道具。
-- 除已发布的 V6 外，其余示例均默认停用；执行前替换为自己的已发布道具。
INSERT INTO f_i_virtual_payment_products(product_key, name, product_id, price_cents, enabled)
VALUES
    ('membership:V6@3900', 'membership:V6@3900', 'fi_v6_year_39', 3900, TRUE),
    ('membership:V7@5900', 'membership:V7@5900', 'fi_v7_year_59', 5900, FALSE),
    ('membership:V8@7900', 'membership:V8@7900', 'fi_v8_year_79', 7900, FALSE),
    ('membership:V9@9900', 'membership:V9@9900', 'fi_v9_year_99', 9900, FALSE),
    ('membership:V7@3500', 'membership:V7@3500', 'fi_v7_upgrade_35', 3500, FALSE),
    ('membership:V8@3500', 'membership:V8@3500', 'fi_v8_upgrade_35', 3500, FALSE),
    ('membership:V8@5500', 'membership:V8@5500', 'fi_v8_upgrade_55', 5500, FALSE),
    ('membership:V9@3500', 'membership:V9@3500', 'fi_v9_upgrade_35', 3500, FALSE),
    ('membership:V9@5500', 'membership:V9@5500', 'fi_v9_upgrade_55', 5500, FALSE),
    ('membership:V9@7500', 'membership:V9@7500', 'fi_v9_upgrade_75', 7500, FALSE),
    ('match_id_unlock@500', 'match_id_unlock@500', 'fi_match_id_5', 500, FALSE),
    ('reflux_subscription:single_match:chengdurongcheng@500', 'reflux_subscription:single_match:chengdurongcheng@500', 'fi_reflux_single_5', 500, FALSE),
    ('reflux_subscription:season_2026:chengdurongcheng@5000', 'reflux_subscription:season_2026:chengdurongcheng@5000', 'fi_reflux_season_50', 5000, FALSE)
ON CONFLICT (product_key) DO NOTHING;
