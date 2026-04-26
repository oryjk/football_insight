CREATE TABLE IF NOT EXISTS f_i_payment_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_no VARCHAR(64) NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES f_i_users(id),
    amount_cents INT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    prepay_id VARCHAR(128),
    transaction_id VARCHAR(128),
    product_type VARCHAR(32) NOT NULL DEFAULT 'membership',
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_f_i_payment_orders_user_id ON f_i_payment_orders(user_id, created_at DESC);

INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'membership_product',
    '{"price_cents":990,"title":"足球洞察会员","subtitle":"解锁全部高级功能","description":"购买后立即升级为 V9 会员，享受全部特权。"}',
    'Membership product config for payment'
)
ON CONFLICT (config_key) DO UPDATE SET
    config_value = EXCLUDED.config_value,
    updated_at = NOW();
