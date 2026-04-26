INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'membership_tier_rules',
    '[
      {"code":"V1","kind":"standard","ticket_watch_poll_interval_seconds":600},
      {"code":"V3","kind":"invite","min_referrals":0,"ticket_watch_poll_interval_seconds":120},
      {"code":"V4","kind":"referral","min_referrals":5,"ticket_watch_poll_interval_seconds":60},
      {"code":"V5","kind":"referral","min_referrals":10,"ticket_watch_poll_interval_seconds":30},
      {"code":"V6","kind":"referral","min_referrals":15,"ticket_watch_poll_interval_seconds":15},
      {"code":"V7","kind":"referral","min_referrals":20,"ticket_watch_poll_interval_seconds":7},
      {"code":"V8","kind":"referral","min_referrals":30,"ticket_watch_poll_interval_seconds":3},
      {"code":"V9","kind":"referral","min_referrals":50,"ticket_watch_poll_interval_seconds":1}
    ]',
    'Membership tier upgrade thresholds and ticket watch polling intervals'
)
ON CONFLICT (config_key) DO UPDATE
SET
    config_value = EXCLUDED.config_value,
    description = EXCLUDED.description,
    updated_at = NOW();
