/** 战术板页面路径：洞察页、首页等入口共用，页面之间不互相引用。 */
export function buildLineupPageUrl(teamId?: number, options: { autosave?: boolean } = {}): string {
  const params: string[] = []
  if (teamId && teamId > 0) params.push(`teamId=${teamId}`)
  // 一次性标记：登录页回跳后新建的页面据此在恢复草稿后自动保存一次。
  if (options.autosave) params.push('autosave=1')
  return params.length ? `/pages/lineup/index?${params.join('&')}` : '/pages/lineup/index'
}

export function parseLineupPageQuery(query: Record<string, unknown> | undefined): { teamId: number; autosave: boolean } {
  const rawTeam = typeof query?.teamId === 'string' ? Number(query.teamId) : 0
  return {
    teamId: Number.isInteger(rawTeam) && rawTeam > 0 ? rawTeam : 0,
    autosave: query?.autosave === '1',
  }
}
