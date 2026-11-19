-- 小程序页面装修配置表：存储每个页面的可选背景图及有序卡片显隐/背景图配置。
-- 未在表中记录的页面使用代码内置默认布局，确保新页面与新卡片零停机自动兜底。

CREATE TABLE IF NOT EXISTS f_i_page_layouts (
    page_key VARCHAR(64) PRIMARY KEY,
    background_image_url TEXT NULL,
    blocks_json JSONB NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    updated_by UUID NULL REFERENCES f_i_admin_users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_f_i_page_layouts_blocks_array CHECK (jsonb_typeof(blocks_json) = 'array')
);
