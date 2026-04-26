UPDATE f_i_system_configs
SET
    config_value = '{"price_cents":4900,"title":"足球洞察会员","subtitle":"解锁全部高级功能","description":"购买后立即升级为 V9 会员，享受全部特权。"}',
    updated_at = NOW()
WHERE config_key = 'membership_product';
