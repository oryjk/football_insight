INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'membership_product',
    '{
      "price_cents":9900,
      "title":"足球洞察会员",
      "subtitle":"V6-V9 回流看板会员",
      "description":"选择适合你的会员档位，购买后立即升级并解锁对应权益。",
      "products":[
        {"target_tier":"V6","price_cents":3900,"title":"V6 回流速览会员","subtitle":"查看 30 分钟内","description":"购买后立即升级为 V6 会员，可查看最近 30 分钟内的回流区域。"},
        {"target_tier":"V7","price_cents":5900,"title":"V7 进阶回流会员","subtitle":"查看 10 分钟内","description":"购买后立即升级为 V7 会员，可查看最近 10 分钟内的回流区域。"},
        {"target_tier":"V8","price_cents":7900,"title":"V8 高阶回流会员","subtitle":"查看 3 分钟内","description":"购买后立即升级为 V8 会员，可查看最近 3 分钟内的回流区域。"},
        {"target_tier":"V9","price_cents":9900,"title":"V9 旗舰回流会员","subtitle":"同 V8 速览权限","description":"购买后立即升级为 V9 会员，最近回流速览权限与 V8 一致。"}
      ]
    }',
    'Membership product catalog for V6 to V9 payment options'
)
ON CONFLICT (config_key) DO UPDATE SET
    config_value = EXCLUDED.config_value,
    description = EXCLUDED.description,
    updated_at = NOW();
