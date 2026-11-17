interface RouteIdentity {
  path: string
  topicId: string
  entryId: string
  commentId: string
  focus: boolean
}

function parseRouteIdentity(rawUrl: string): RouteIdentity {
  const [rawPath, rawQuery = ''] = rawUrl.split('?')
  const path = rawPath.startsWith('/') ? rawPath : `/${rawPath}`
  const params = new URLSearchParams(rawQuery)
  return {
    path,
    topicId: params.get('topicId') ?? '',
    entryId: params.get('entryId') ?? '',
    commentId: params.get('commentId') ?? '',
    focus: params.get('focus') === '1',
  }
}

function isSameTarget(a: RouteIdentity, b: RouteIdentity): boolean {
  return (
    a.path === b.path
    && a.topicId === b.topicId
    && a.entryId === b.entryId
    && a.commentId === b.commentId
  )
}

function currentPageFullUrl(page: { route?: string; options?: Record<string, unknown>; $page?: { fullPath?: string } }): string {
  if (typeof page.$page?.fullPath === 'string' && page.$page.fullPath) {
    return page.$page.fullPath
  }
  const route = page.route ? `/${page.route.replace(/^\/+/, '')}` : ''
  const options = page.options ?? {}
  const entries = Object.entries(options)
    .filter(([, value]) => typeof value === 'string' && value.length > 0)
    .map(([key, value]) => `${key}=${encodeURIComponent(String(value))}`)
  return entries.length ? `${route}?${entries.join('&')}` : route
}

/**
 * 在电报详情与评论详情页间安全跳转：
 * - 若上一页正是目标页（且无需 focus=1 自动弹键盘），优先直接 navigateBack 避免环形压栈；
 * - 若当前小程序页面栈接近 10 层上限（>= 9），切换为 redirectTo 防止 wx.navigateTo 超限失败。
 */
export function navigateInTelegraphStack(url: string): void {
  const pages = typeof getCurrentPages === 'function' ? getCurrentPages() : []
  if (pages.length >= 2) {
    const previous = pages[pages.length - 2] as { route?: string; options?: Record<string, unknown>; $page?: { fullPath?: string } }
    const target = parseRouteIdentity(url)
    const prev = parseRouteIdentity(currentPageFullUrl(previous))
    if (!target.focus && isSameTarget(target, prev)) {
      uni.navigateBack({ delta: 1 })
      return
    }
  }
  if (pages.length >= 9) {
    uni.redirectTo({ url })
    return
  }
  uni.navigateTo({ url })
}
