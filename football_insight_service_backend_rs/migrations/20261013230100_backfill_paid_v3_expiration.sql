-- 单独事务执行，避免持有 ALTER TABLE 锁时等待正在结算/退款的订单。
DO $$
DECLARE
    candidate RECORD;
    payment RECORD;
    old_grant JSONB;
    new_grant JSONB;
    duration INTEGER;
BEGIN
    -- 先锁住所有候选订单，再持有账号锁；避免多笔旧订单与并发退款互相等待。
    PERFORM p.order_no
    FROM f_i_payment_orders p JOIN f_i_virtual_payment_orders v USING(order_no)
    WHERE p.product_type='membership:V3' AND p.status='paid'
      AND v.grant_after->>'tier'='V3' AND v.grant_after->>'expires_at' IS NULL
    ORDER BY p.user_id, p.paid_at DESC, p.order_no DESC
    FOR UPDATE OF p;
    FOR candidate IN
        SELECT p.order_no, p.user_id
        FROM f_i_payment_orders p JOIN f_i_virtual_payment_orders v USING(order_no)
        WHERE p.product_type='membership:V3' AND p.status='paid'
          AND v.grant_after->>'tier'='V3' AND v.grant_after->>'expires_at' IS NULL
        ORDER BY p.user_id, p.paid_at DESC, p.order_no DESC
    LOOP
        -- 与发货/退款相同的锁顺序：订单、账号、权益元数据。
        SELECT status, paid_at INTO payment FROM f_i_payment_orders
        WHERE order_no=candidate.order_no FOR UPDATE;
        IF payment.status <> 'paid' OR payment.paid_at IS NULL THEN CONTINUE; END IF;
        PERFORM id FROM f_i_users WHERE id=candidate.user_id FOR UPDATE;
        SELECT grant_after, membership_duration_months INTO old_grant, duration
        FROM f_i_virtual_payment_orders WHERE order_no=candidate.order_no FOR UPDATE;
        IF old_grant->>'tier' IS DISTINCT FROM 'V3'
            OR old_grant->>'expires_at' IS NOT NULL THEN CONTINUE; END IF;

        new_grant := jsonb_set(old_grant, '{expires_at}',
            to_jsonb(payment.paid_at + make_interval(months => duration)), TRUE);
        -- 只有当前权益仍等于旧订单授予结果才补到期日，保留后续购买/人工调整。
        UPDATE f_i_users SET membership_expires_at=(new_grant->>'expires_at')::timestamptz,
            updated_at=NOW()
        WHERE id=candidate.user_id
          AND jsonb_build_object('tier',membership_tier,'expires_at',membership_expires_at)=old_grant;
        UPDATE f_i_virtual_payment_orders SET grant_after=new_grant
        WHERE order_no=candidate.order_no;
        -- 同步退款链，后续订单退款时不复活旧的无期限 V3。
        UPDATE f_i_virtual_payment_orders successor SET grant_before=new_grant
        FROM f_i_payment_orders p
        WHERE successor.order_no=p.order_no AND p.user_id=candidate.user_id
          AND p.status='paid' AND successor.order_no<>candidate.order_no
          AND (p.paid_at, p.order_no) > (payment.paid_at, candidate.order_no)
          AND successor.grant_before=old_grant;
    END LOOP;
END $$;
