ALTER TABLE f_i_virtual_payment_products
    ADD COLUMN membership_duration_months INTEGER NOT NULL DEFAULT 12
    CHECK (membership_duration_months BETWEEN 1 AND 1200);
COMMENT ON COLUMN f_i_virtual_payment_products.membership_duration_months IS '会员商品有效期（月），12 表示一个自然年；其他商品不使用此字段';

-- 默认值兼容滚动部署中的旧后端；历史订单固定为原先一年规则。
ALTER TABLE f_i_virtual_payment_orders
    ADD COLUMN membership_duration_months INTEGER NOT NULL DEFAULT 12
    CHECK (membership_duration_months BETWEEN 1 AND 1200);
COMMENT ON COLUMN f_i_virtual_payment_orders.membership_duration_months IS '下单时会员有效期快照；后续修改商品不改变已创建订单';
