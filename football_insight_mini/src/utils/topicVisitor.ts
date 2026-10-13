const VISITOR_KEY = 'fi_topic_visitor_id'
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i

/** Anonymous device marker; it is not an authentication credential. */
export function topicVisitorId(): string | null {
  try {
    const existing = uni.getStorageSync(VISITOR_KEY)
    if (typeof existing === 'string' && UUID.test(existing)) return existing
    const id = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, key => {
      const value = Math.floor(Math.random() * 16)
      return (key === 'x' ? value : (value & 3) | 8).toString(16)
    })
    uni.setStorageSync(VISITOR_KEY, id)
    return id
  } catch { return null }
}
