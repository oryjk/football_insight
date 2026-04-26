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

  return '现在就能为你的主队站队助力，并把页面转发给朋友继续拉票。'
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
  favoriteTeam: FavoriteTeamLike,
): string {
  if (!detail) {
    return ''
  }

  if (detail.viewer.has_supported) {
    return '这场比赛你已经完成一次助力，接下来更适合转发拉票。'
  }

  if (!favoriteTeam) {
    return '先去首页关注一支主队，回到这里才能为它助力。'
  }

  if (detail.support_window_status === 'closed') {
    return '比赛已经开始，当前只能查看这场比赛的最终助力结果。'
  }

  return '当前助力只允许投给你关注的主队，每场比赛只能投一次。'
}

export function resolveSupportVoteButtonText(
  detail: Pick<SupportMatchDetail, 'viewer' | 'support_window_status'> | null,
  favoriteTeam: FavoriteTeamLike,
  hasAccessToken: boolean,
): string {
  if (!detail) {
    return '为主队助力'
  }

  if (!hasAccessToken) {
    return '登录后为主队助力'
  }

  if (detail.viewer.has_supported) {
    return '你已完成本场助力'
  }

  if (!favoriteTeam) {
    return '先去首页关注主队'
  }

  if (detail.support_window_status === 'closed') {
    return '本场助力已关闭'
  }

  return `为 ${favoriteTeam.team_name} 助力`
}
