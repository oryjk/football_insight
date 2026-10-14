import type { SupportMatchDetail } from '../types/support'

/** 当前已助力球队可点击取消；对方需先取消现有助力才能重新选择。 */
export function resolveSupportVoteState(
  match: SupportMatchDetail | null,
  teamId: number,
  submittingTeamId: number | null,
  now: number,
  changingFavorite = false,
): { label: string; disabled: boolean; supported: boolean } {
  const supported = !!match?.viewer.has_supported && match.viewer.supported_team_id === teamId
  const kickoff = Date.parse(match?.kickoff_at ?? '')
  const open = !!match && match.support_window_status === 'open'
    && Number.isFinite(kickoff) && kickoff > now
  const busy = submittingTeamId !== null || changingFavorite
  if (submittingTeamId === teamId) return { label: supported ? '取消中' : '提交中', disabled: true, supported }
  if (supported) return { label: '已助力', disabled: !open || busy, supported: true }
  return {
    label: !open && match?.support_window_status !== 'locked' ? '已结束' : '助力',
    disabled: !open || !match?.viewer.can_support || match.viewer.has_supported || busy,
    supported: false,
  }
}
