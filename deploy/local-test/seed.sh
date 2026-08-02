#!/usr/bin/env bash
set -euo pipefail

API_URL=${LOCAL_TEST_API_URL:-http://127.0.0.1:18092}
PG_CONTAINER=football-insight-local-test-postgres

login_json=$(curl -fsS "$API_URL/api/v1/admin/auth/login" \
    -H 'Content-Type: application/json' \
    -d '{"username":"owner","password":"FootballTest2026!"}')
token=$(jq -r '.access_token' <<<"$login_json")
test -n "$token" && test "$token" != "null"

auth_header=( -H "Authorization: Bearer $token" )

find_user_id() {
    local account="$1"
    curl -fsS "$API_URL/api/v1/admin/users?query=$account&page=1&page_size=10" "${auth_header[@]}" \
        | jq -r --arg account "$account" '.items[] | select(.account_identifier == $account) | .id' \
        | head -n 1
}

create_user_if_missing() {
    local account="$1"
    local display_name="$2"
    local tier="$3"
    local id
    id=$(find_user_id "$account")
    if [[ -n "$id" ]]; then
        printf '%s' "$id"
        return
    fi
    curl -fsS "$API_URL/api/v1/admin/users" \
        "${auth_header[@]}" \
        -H 'Content-Type: application/json' \
        -d "{\"account_identifier\":\"$account\",\"display_name\":\"$display_name\",\"password\":\"TestUser2026!\",\"membership_tier\":\"$tier\"}" \
        | jq -r '.id'
}

user_a=$(create_user_if_missing 'local-test-fan-001' '本地测试球迷一' 'V3')
user_b=$(create_user_if_missing 'local-test-fan-002' '本地测试球迷二' 'V1')
test -n "$user_a" && test -n "$user_b"

user_b_status=$(curl -fsS "$API_URL/api/v1/admin/users/$user_b" "${auth_header[@]}" | jq -r '.status')
if [[ "$user_b_status" == "active" ]]; then
    curl -fsS -X POST "$API_URL/api/v1/admin/users/$user_b/disable" \
        "${auth_header[@]}" \
        -H 'Content-Type: application/json' \
        -d '{"reason":"本地测试禁用恢复流程"}' >/dev/null
fi

docker exec -i "$PG_CONTAINER" psql -v ON_ERROR_STOP=1 -U football_test -d football_insight_test \
    -v user_a="$user_a" -v user_b="$user_b" <<'SQL'
INSERT INTO f_i_invite_codes (id, code, is_active, used_by_user_id, used_at, created_at)
VALUES ('10000000-0000-0000-0000-000000000001', 'LOCAL-TEST-REF', false, :'user_b'::uuid, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET used_by_user_id = EXCLUDED.used_by_user_id, used_at = EXCLUDED.used_at;

INSERT INTO f_i_user_referrals (id, referrer_user_id, referred_user_id, referral_invite_code, created_at)
VALUES ('20000000-0000-0000-0000-000000000001', :'user_a'::uuid, :'user_b'::uuid, 'LOCAL-TEST-REF', NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO f_i_user_activity_snapshots (user_id, last_login_at, last_active_at, last_active_page_key, updated_at)
VALUES (:'user_a'::uuid, NOW() - INTERVAL '2 hours', NOW() - INTERVAL '15 minutes', 'live-overview', NOW())
ON CONFLICT (user_id) DO UPDATE SET last_login_at = EXCLUDED.last_login_at, last_active_at = EXCLUDED.last_active_at, last_active_page_key = EXCLUDED.last_active_page_key, updated_at = NOW();

INSERT INTO f_i_payment_orders (id, order_no, user_id, amount_cents, status, product_type, paid_at, created_at, updated_at)
VALUES ('30000000-0000-0000-0000-000000000001', 'LOCAL-TEST-ORDER-001', :'user_a'::uuid, 990, 'paid', 'membership', NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day', NOW())
ON CONFLICT (order_no) DO UPDATE SET user_id = EXCLUDED.user_id, status = EXCLUDED.status, updated_at = NOW();

INSERT INTO f_i_user_reflux_subscriptions (id, user_id, plan_code, scope, team_code, season, order_no, starts_at, expires_at, status, created_at, updated_at)
VALUES ('40000000-0000-0000-0000-000000000001', :'user_a'::uuid, 'season_2026', 'season', 'global', 2026, 'LOCAL-TEST-ORDER-001', NOW() - INTERVAL '1 day', NOW() + INTERVAL '120 days', 'active', NOW() - INTERVAL '1 day', NOW())
ON CONFLICT (id) DO UPDATE SET status = EXCLUDED.status, expires_at = EXCLUDED.expires_at, updated_at = NOW();

INSERT INTO f_i_user_device_tokens (user_id, device_token, platform, created_at, updated_at)
VALUES (:'user_a'::uuid, 'local-test-device-token-abcdef1234567890', 'fcm', NOW() - INTERVAL '3 days', NOW())
ON CONFLICT (device_token) DO UPDATE SET user_id = EXCLUDED.user_id, updated_at = NOW();
SQL

echo "Seeded local users: $user_a, $user_b"
