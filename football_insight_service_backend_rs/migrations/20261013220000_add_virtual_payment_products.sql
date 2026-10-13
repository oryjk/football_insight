CREATE TABLE f_i_virtual_payment_products (
    product_key TEXT PRIMARY KEY,
    name TEXT NOT NULL CHECK (length(btrim(name)) > 0),
    product_id TEXT NOT NULL CHECK (product_id = btrim(product_id) AND length(product_id) > 0),
    price_cents INTEGER NOT NULL CHECK (price_cents > 0),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    remark TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT virtual_product_key_matches_price CHECK (
        product_key ~ '^(membership:V[6-9]|match_id_unlock|reflux_subscription:[^:@]+:[^:@]+)@[1-9][0-9]*$'
        AND split_part(product_key, '@', 2) = price_cents::text
    )
);

COMMENT ON TABLE f_i_virtual_payment_products IS '微信虚拟支付道具映射，下单实时读取；密钥仍由环境变量管理';
COMMENT ON COLUMN f_i_virtual_payment_products.product_key IS '业务规格@实付金额（分），含会员补差价规格';
COMMENT ON COLUMN f_i_virtual_payment_products.product_id IS '微信后台已发布的道具 ID';
COMMENT ON COLUMN f_i_virtual_payment_products.price_cents IS '必须与业务结算价和微信后台道具价格一致，单位分';
COMMENT ON COLUMN f_i_virtual_payment_products.enabled IS '仅控制新增购买，不影响历史订单结算';

CREATE FUNCTION f_i_virtual_payment_products_touch_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER virtual_payment_products_updated_at
BEFORE UPDATE ON f_i_virtual_payment_products
FOR EACH ROW EXECUTE FUNCTION f_i_virtual_payment_products_touch_updated_at();

-- 已在微信现网发布并用于安卓真机调起的商品，不添加尚未发布的示例商品。
INSERT INTO f_i_virtual_payment_products(product_key, name, product_id, price_cents, enabled)
VALUES ('membership:V6@3900', 'V6 年会员', 'fi_v6_year_39', 3900, TRUE);
