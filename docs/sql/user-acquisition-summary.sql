-- READ ONLY. Requires 20261018120000_add_user_acquisitions.sql to be applied by the owner.
-- D1+/D7+ mean last successful login >= registration + 24/168 hours, NOT exact-day retention.
-- record_user_login keeps only the latest login, so strict day-1/day-7 cohorts cannot be reconstructed.
-- Eligible denominators exclude accounts younger than 24/168 hours at as_of.
-- A later refund still counts as "ever paid"; pending/unpaid orders and future paid_at do not.
-- Change only params to restrict the REGISTRATION cohort; timestamps are absolute instants.
WITH params AS (
    SELECT CURRENT_TIMESTAMP AS as_of,
           NULL::timestamptz AS registered_from,
           NULL::timestamptz AS registered_before
), cohort AS (
    SELECT a.user_id, a.src, a.cid, u.created_at, s.last_login_at, p.as_of
    FROM f_i_user_acquisitions AS a
    JOIN f_i_users AS u ON u.id = a.user_id
    LEFT JOIN f_i_user_activity_snapshots AS s ON s.user_id = a.user_id
    CROSS JOIN params AS p
    WHERE u.created_at <= p.as_of
      AND (p.registered_from IS NULL OR u.created_at >= p.registered_from)
      AND (p.registered_before IS NULL OR u.created_at < p.registered_before)
)
SELECT c.src, c.cid,
       COUNT(*) AS registered_users,
       COUNT(*) FILTER (WHERE c.created_at + INTERVAL '24 hours' <= c.as_of) AS d1_eligible_users,
       COUNT(*) FILTER (WHERE c.last_login_at >= c.created_at + INTERVAL '24 hours'
                         AND c.last_login_at <= c.as_of) AS d1_plus_return_users,
       COUNT(*) FILTER (WHERE c.created_at + INTERVAL '168 hours' <= c.as_of) AS d7_eligible_users,
       COUNT(*) FILTER (WHERE c.last_login_at >= c.created_at + INTERVAL '168 hours'
                         AND c.last_login_at <= c.as_of) AS d7_plus_return_users,
       COUNT(*) FILTER (WHERE EXISTS (
           SELECT 1 FROM f_i_payment_orders AS orders
           WHERE orders.user_id = c.user_id
             AND orders.status IN ('paid', 'refunded')
             AND orders.paid_at IS NOT NULL
             AND orders.paid_at <= c.as_of
       )) AS paid_users
FROM cohort AS c
GROUP BY c.src, c.cid
ORDER BY registered_users DESC, c.src, c.cid NULLS FIRST;
