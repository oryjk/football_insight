/** 战术板页面路径：洞察页、首页等入口共用，页面之间不互相引用。 */
export function buildLineupPageUrl(teamId?: number): string {
  return teamId && teamId > 0 ? `/pages/lineup/index?teamId=${teamId}` : '/pages/lineup/index'
}
