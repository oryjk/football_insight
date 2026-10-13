CREATE TABLE f_i_virtual_payment_orders (
    order_no VARCHAR(64) PRIMARY KEY REFERENCES f_i_payment_orders(order_no),
    open_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    amount_cents INT NOT NULL CHECK (amount_cents > 0),
    wx_order_id TEXT UNIQUE,
    delivery_acknowledged BOOLEAN NOT NULL DEFAULT FALSE,
    grant_before JSONB,
    grant_after JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_checked_at TIMESTAMPTZ
);
CREATE TABLE f_i_virtual_payment_refunds (
    refund_id TEXT PRIMARY KEY,
    order_no VARCHAR(64) NOT NULL REFERENCES f_i_virtual_payment_orders(order_no),
    amount_cents INT NOT NULL CHECK (amount_cents > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
