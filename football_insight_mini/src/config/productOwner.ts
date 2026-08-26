// 产品负责人账号：唯一可见「我的 → 设置」入口的账号。
// 与后端 MINI_REVIEW_CONTROL_USER_IDS 白名单保持一致（football 用户 id 为 UUID）。
// 可通过 VITE_PRODUCT_OWNER_USER_ID 覆盖；默认即管理员账号。
export const PRODUCT_OWNER_USER_ID = (import.meta.env.VITE_PRODUCT_OWNER_USER_ID ?? '').trim()
  || '97f5ca05-6f69-4346-ba69-1b22970fc4f6'
