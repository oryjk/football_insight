-- Operator-only seed, never run by the application or migrations.
-- Apply 20261016130000 first. Match the published WeChat IDs and prices before activation.
-- Start disabled as required by the P1 design; activation is a separate operator action.
INSERT INTO f_i_virtual_payment_products
 (product_key,name,product_id,price_cents,enabled,kind,grant_spec,eligibility,requires_capability,
  title,subtitle,description,group_key,sort_order,membership_duration_months)
VALUES
 ('membership:V4@900','V4 场次体验卡','fi_v6_day_9',900,false,'membership',
  '{"tier":"V4","expiry":{"rule":"end_of_next_day","tz":"Asia/Shanghai"}}',
  '{"max_current_tier":"V3"}','catalog-v2',
  'V4 场次体验卡','次日 23:59 前有效','支付后立即升级为 V4，余票监控刷新提速至 60 秒。','pass',10,1);

INSERT INTO f_i_virtual_payment_products
 (product_key,name,product_id,price_cents,enabled,kind,grant_spec,eligibility,requires_capability,
  title,subtitle,description,group_key,sort_order,membership_duration_months)
VALUES
 ('membership:V5@1900','V5 场次体验卡','fi_v9_day_19',1900,false,'membership',
  '{"tier":"V5","expiry":{"rule":"end_of_next_day","tz":"Asia/Shanghai"}}',
  '{"max_current_tier":"V3"}','catalog-v2',
  'V5 场次体验卡','次日 23:59 前有效','支付后立即升级为 V5，余票监控刷新提速至 30 秒。','pass',20,1);
