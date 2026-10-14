import type { SupportMatchDetail, SupportMatchTeam } from '../../types/support'
import { traceRoundRect } from '../../utils/canvasPoster'

export const SUPPORT_POSTER_CANVAS_ID = 'supportPosterCanvas'
export const SUPPORT_POSTER_SIZE = { width: 960, height: 1320 } as const

export function buildSupportPosterHeadline(
  detail: Pick<SupportMatchDetail, 'home_team' | 'away_team'>,
  supportedTeam: Pick<SupportMatchTeam, 'team_id' | 'team_name'> | null,
): string {
  if (supportedTeam) {
    const rivalTeam = supportedTeam.team_id === detail.home_team.team_id ? detail.away_team : detail.home_team
    return `我已为${supportedTeam.team_name}助力，来把${rivalTeam.team_name}压过去`
  }

  return `${detail.home_team.team_name} 和 ${detail.away_team.team_name} 的热度对抗已经打响`
}

/** 拉票海报绘制（颜色为海报插画色，decorative，不走 design token）。 */
export function drawSupportPoster(
  context: UniApp.CanvasContext,
  detail: SupportMatchDetail,
  supportedTeam: SupportMatchTeam | null,
): void {
  const { width, height } = SUPPORT_POSTER_SIZE

  context.setFillStyle('#f5f0e8')
  context.fillRect(0, 0, width, height)

  context.setFillStyle('#ffffff')
  traceRoundRect(context, 44, 44, width - 88, height - 88, 40)
  context.fill()

  context.setFillStyle('#121212')
  context.setFontSize(34)
  context.fillText('足球洞察 · 赛前助力', 96, 132)

  context.setFontSize(52)
  context.fillText(detail.home_team.team_name, 96, 244)
  context.fillText('VS', width / 2 - 26, 326)
  context.fillText(detail.away_team.team_name, 96, 408)

  context.setFillStyle('#6b707b')
  context.setFontSize(28)
  context.fillText(`${detail.match_date} ${detail.match_time} · 第 ${detail.round_number} 轮`, 96, 486)

  context.setFillStyle('#dc2626')
  context.fillRect(96, 560, Math.max(40, (width - 192) * (detail.home_team.support_share_pct / 100)), 28)
  context.setFillStyle('#2563eb')
  context.fillRect(96, 616, Math.max(40, (width - 192) * (detail.away_team.support_share_pct / 100)), 28)

  context.setFillStyle('#121212')
  context.setFontSize(32)
  context.fillText(`${detail.home_team.team_name} ${detail.home_team.support_count}`, 96, 724)
  context.fillText(`${detail.away_team.team_name} ${detail.away_team.support_count}`, 96, 786)

  context.setFillStyle('#444b57')
  context.setFontSize(40)
  context.fillText(buildSupportPosterHeadline(detail, supportedTeam), 96, 926)

  context.setFillStyle('#6b707b')
  context.setFontSize(28)
  context.fillText('打开小程序，进入这场比赛继续站队助力。', 96, 1004)
  context.fillText('主客队任选，开球前可取消重新选择。', 96, 1050)
}
