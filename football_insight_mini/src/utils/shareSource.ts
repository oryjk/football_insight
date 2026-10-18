/** Attribution must never prevent a public share or replace an existing campaign. */
export function buildShareQuery(cid: string): string {
  return /^[a-z0-9_-]{1,64}$/.test(cid) ? `src=share&cid=${cid}` : ''
}

export function buildSharePath(path: string, cid: string): string {
  const source = buildShareQuery(cid)
  if (!path || !source) return path
  const hashIndex = path.indexOf('#')
  const base = hashIndex < 0 ? path : path.slice(0, hashIndex)
  const hash = hashIndex < 0 ? '' : path.slice(hashIndex)
  if (!base) return path
  const queryIndex = base.indexOf('?')
  if (queryIndex >= 0) {
    const hasSource = base.slice(queryIndex + 1).split('&').some(pair => {
      try {
        const key = decodeURIComponent(pair.split('=')[0]!)
        return key === 'src' || key === 'cid'
      } catch { return false }
    })
    if (hasSource) return path
  }
  const separator = queryIndex < 0 ? '?' : /[?&]$/.test(base) ? '' : '&'
  return `${base}${separator}${source}${hash}`
}
