/** A stale dev compiler may not yet know the Vite define. Diagnostics must never block startup. */
export function getDevBuildInfo(): { revision: string; builtAt: string } {
  return typeof __FOOTBALL_BUILD_INFO__ === 'undefined'
    ? { revision: 'unavailable', builtAt: 'unavailable' }
    : __FOOTBALL_BUILD_INFO__
}
