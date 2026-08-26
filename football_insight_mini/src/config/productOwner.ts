// 产品负责人账号：唯一可见「我的 → 设置」入口的账号。
// 与后端 MINI_REVIEW_CONTROL_USER_IDS 白名单保持一致（football 用户 id 为 UUID）。
// 通过 VITE_PRODUCT_OWNER_USER_ID 配置；为空时设置入口对所有人隐藏。
export const PRODUCT_OWNER_USER_ID = (import.meta.env.VITE_PRODUCT_OWNER_USER_ID ?? '').trim()
