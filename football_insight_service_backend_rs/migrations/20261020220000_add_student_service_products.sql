-- New service mappings are managed atomically with student feature configuration.
-- No unconfigured or unpublished payment products are enabled by migration.
ALTER TABLE f_i_virtual_payment_products
    DROP CONSTRAINT virtual_product_key_matches_price,
    ADD CONSTRAINT virtual_product_key_matches_price CHECK (
        product_key ~ '^(membership:V[3-9]|match_id_unlock|reflux_subscription:[^:@]+:[^:@]+|student_service:(certificate_renewal|tool_upgrade))@[1-9][0-9]*$'
        AND split_part(product_key, '@', 2) = price_cents::text
        AND (product_key NOT LIKE 'student_service:%'
            OR (product_key = 'student_service:certificate_renewal@9900' AND price_cents = 9900 AND kind = 'student_service')
            OR (product_key = 'student_service:tool_upgrade@69900' AND price_cents = 69900 AND kind = 'student_service'))
    );

CREATE INDEX IF NOT EXISTS f_i_payment_orders_student_user_created_idx
    ON f_i_payment_orders(user_id, created_at DESC, order_no DESC)
    WHERE product_type IN ('student_service:certificate_renewal', 'student_service:tool_upgrade');

-- Uploaded owner-provided QR image; keep both services disabled until descriptions
-- and matching published WeChat product IDs are configured by an administrator.
INSERT INTO f_i_system_configs(config_key, config_value, description)
VALUES ('student_features', '{"contact_qr_url":"https://oryjk.cn:82/minio/football-insight/summary/student/contact-wechat-740aed5038f2113f.jpg","services":[{"code":"certificate_renewal","title":"证书续期","description":"","price_cents":9900,"unit":"年","enabled":false,"product_id":""},{"code":"tool_upgrade","title":"工具升级","description":"","price_cents":69900,"unit":"套","enabled":false,"product_id":""}]}', '学员功能说明、联系微信与购买配置')
ON CONFLICT(config_key) DO NOTHING;
