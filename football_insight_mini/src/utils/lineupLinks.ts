/** 战术板页面路径：洞察页、首页等入口共用，页面之间不互相引用。 */
export function buildLineupPageUrl(teamId?: number, options: { autosave?: boolean; resumeAction?: 'draft' | 'publish' } = {}): string {
  const params: string[] = []
  if (teamId && teamId > 0) params.push(`teamId=${teamId}`)
  // 一次性标记：登录页回跳后新建的页面据此在恢复草稿后自动保存一次。
  if (options.resumeAction) params.push(`resumeAction=${options.resumeAction}`)
  else if (options.autosave) params.push('autosave=1')
  return params.length ? `/pages/lineup/index?${params.join('&')}` : '/pages/lineup/index'
}

export function parseLineupPageQuery(query: Record<string, unknown> | undefined): { teamId: number; autosave: boolean; resumeAction: 'draft' | 'publish' | null } {
  const rawTeam = typeof query?.teamId === 'string' ? Number(query.teamId) : 0
  return {
    teamId: Number.isInteger(rawTeam) && rawTeam > 0 ? rawTeam : 0,
    autosave: query?.autosave === '1',
    // 旧版本 autosave 始终只恢复私密草稿。
    resumeAction: query?.resumeAction === 'publish' ? 'publish' : query?.resumeAction === 'draft' || query?.autosave === '1' ? 'draft' : null,
  }
}


export function validLineupCode(code: unknown): code is string {
  return typeof code === 'string' && /^[a-zA-Z0-9]{8}$/.test(code)
}
export function buildLineupViewUrl(code: string, shared = false): string {
  if (!validLineupCode(code)) return ''
  return `/pages/lineup/view?code=${code}${shared ? '&src=share&cid=lineup' : ''}`
}
export function buildLineupListUrl(teamId: number, sort: 'latest' | 'likes' = 'latest'): string {
  return `/pages/lineup/list?teamId=${teamId}${sort === 'likes' ? '&sort=likes' : ''}`
}
export function parseLineupShareCode(query: Record<string, unknown> | undefined): string {
  if (validLineupCode(query?.code)) return query!.code as string
  if (typeof query?.scene !== 'string') return ''
  try {
    const scene = decodeURIComponent(query.scene)
    const values = Object.fromEntries(scene.split('&').map(pair => pair.split('=')))
    return values.s === 'share' && values.c === 'lineup' && validLineupCode(values.l) ? values.l : ''
  } catch { return '' }
}

export function parseLineupListQuery(query: Record<string, unknown> | undefined): { teamId: number; sort: 'latest' | 'likes' } {
  return { teamId: parseLineupPageQuery(query).teamId, sort: query?.sort === 'likes' ? 'likes' : 'latest' }
}
