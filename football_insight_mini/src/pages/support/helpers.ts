import type { SupportMatchDetail, SupportMatchTeam } from '../../types/support'

type SupportMatchLike = Pick<SupportMatchDetail, 'support_window_status' | 'status' | 'countdown_seconds'>

type FavoriteTeamLike = Pick<SupportMatchTeam, 'team_name'> | null

export function resolveSupportHeroBadge(detail: SupportMatchLike | null): string {
  if (!detail) {
    return 'MVP'
  }

  if (detail.support_window_status === 'closed') {
    return '已截止'
  }

  return '助力进行中'
}

export function resolveSupportHeroSummary(detail: SupportMatchLike | null): string {
  if (!detail) {
    return '赛前开放助力，比赛开始后自动截止。'
  }

  if (detail.support_window_status === 'closed') {
    return detail.status === 'finished'
      ? '这场比赛的助力已经封盘，可以直接看双方最终热度结果。'
      : '比赛已经开始，赛前助力入口已自动关闭。'
  }

  return '任选主队或客队助力，并把页面转发给朋友继续拉票。'
}

export function resolveSupportWindowLabel(detail: SupportMatchLike | null): string {
  if (!detail) {
    return ''
  }

  return detail.support_window_status === 'closed' ? '助力关闭' : '助力开放'
}

export function resolveSupportCountdownLabel(detail: SupportMatchLike | null): string {
  if (!detail) {
    return ''
  }

  if (detail.support_window_status === 'closed') {
    return detail.status === 'finished' ? '比赛已完赛' : '比赛已开始'
  }

  const totalHours = Math.floor(detail.countdown_seconds / 3600)
  const minutes = Math.floor((detail.countdown_seconds % 3600) / 60)
  const seconds = detail.countdown_seconds % 60
  return `距开赛 ${totalHours} 小时 ${minutes} 分 ${seconds} 秒`
}

export function resolveSupportCountdownSeconds(
  kickoffAt: string,
  nowMs = Date.now(),
): number {
  const kickoffMs = new Date(kickoffAt).getTime()
  if (Number.isNaN(kickoffMs)) {
    return 0
  }

  return Math.max(0, Math.floor((kickoffMs - nowMs) / 1000))
}

export function resolveSupportStanceDescription(
  detail: Pick<SupportMatchDetail, 'viewer' | 'support_window_status'> | null,
  _favoriteTeam: FavoriteTeamLike,
): string {
  if (!detail) {
    return ''
  }

  if (detail.viewer.has_supported) {
    return detail.support_window_status === 'closed'
      ? '助力已截止，可以转发本场助力结果。'
      : '已助力，开球前可点击对应按钮取消，并重新选择球队。'
  }

  if (detail.support_window_status === 'closed') {
    return '比赛已经开始，当前只能查看这场比赛的最终助力结果。'
  }

  return '主队和客队均可助力，开球前可取消，每场同一时间只能保留一队的助力。'
}

export function resolveSupportedTeam(
  detail: Pick<SupportMatchDetail, 'viewer' | 'home_team' | 'away_team'> | null,
): SupportMatchTeam | null {
  if (!detail?.viewer.has_supported) return null
  return [detail.home_team, detail.away_team].find(team => team.team_id === detail.viewer.supported_team_id) ?? null
}

export function resolveSupportVoteButtonText(
  detail: Pick<SupportMatchDetail, 'viewer' | 'support_window_status'> | null,
  favoriteTeam: FavoriteTeamLike,
  hasAccessToken: boolean,
): string {
  if (!detail) {
    return '选择球队助力'
  }

  if (!hasAccessToken) {
    return '登录后助力'
  }

  if (detail.viewer.has_supported) {
    return '你已完成本场助力'
  }

  if (!favoriteTeam) {
    return '选择球队助力'
  }

  if (detail.support_window_status === 'closed') {
    return '本场助力已关闭'
  }

  return `为 ${favoriteTeam.team_name} 助力`
}
