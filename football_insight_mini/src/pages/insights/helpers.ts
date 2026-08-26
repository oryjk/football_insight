import type {
  AssistContribution,
  OpponentContribution,
  PlayerContribution,
} from '../../types/insight'

/** 贡献结构的统一展示行模型：四张 breakdown 卡共用一个组件渲染。 */
export interface InsightContributionRow {
  key: string
  instanceKey: string
  name: string
  note: string
  avatar: string | null
  avatarMode: 'fit' | 'fill'
  share: number
  variant: 'ink' | 'red' | 'green' | 'danger'
}

export const INSIGHTS_PREVIEW_TEAMS = ['成都蓉城', '上海申花', '武汉三镇', '山东泰山']
export const INSIGHTS_PREVIEW_GOAL_AGAINST_ROWS = [
  { name: '青岛西海岸', width: '76%' },
  { name: '深圳新鹏城', width: '58%' },
  { name: '重庆铜梁龙', width: '34%' },
]
export const INSIGHTS_PREVIEW_GOAL_PLAYER_ROWS = [
  { name: '席尔瓦', width: '54%' },
  { name: '费利佩', width: '30%' },
  { name: '其他 / 未归因', width: '18%' },
]
export const INSIGHTS_PREVIEW_ASSIST_ROWS = [
  { name: '费利佩', width: '64%' },
  { name: '席尔瓦', width: '46%' },
  { name: '其他 / 未归因', width: '40%' },
]

export function getVisibleInsightContributions<T>(items: T[], expanded: boolean): T[] {
  return expanded ? items : items.slice(0, 3)
}

export function shouldShowInsightToggle<T>(items: T[]): boolean {
  return items.length > 3
}

export function formatInsightShare(value: number): string {
  return `${(value * 100).toFixed(1)}%`
}

export function insightBarWidth(value: number): string {
  if (value <= 0) {
    return '0%'
  }

  return `${Math.max(value * 100, 8)}%`
}

function opponentRowKey(item: OpponentContribution): string {
  return `team-${item.opponent_team_id}`
}

function playerRowKey(item: PlayerContribution): string {
  return `player-${item.player_id ?? item.player_name}`
}

function assistRowKey(item: AssistContribution): string {
  return `assist-${item.player_id ?? item.player_name}`
}

export function buildOpponentContributionRows(
  scope: string,
  teamId: number | null,
  items: OpponentContribution[],
  variant: InsightContributionRow['variant'],
): InsightContributionRow[] {
  return items.map(item => ({
    key: opponentRowKey(item),
    instanceKey: `${teamId ?? 'none'}-${scope}-${opponentRowKey(item)}`,
    name: item.opponent_team_name,
    note: `${item.goals} 球`,
    avatar: item.opponent_avatar_storage_url,
    avatarMode: 'fit' as const,
    share: item.share,
    variant,
  }))
}

export function buildPlayerContributionRows(
  scope: string,
  teamId: number | null,
  items: PlayerContribution[],
  variant: InsightContributionRow['variant'],
): InsightContributionRow[] {
  return items.map(item => ({
    key: playerRowKey(item),
    instanceKey: `${teamId ?? 'none'}-${scope}-${playerRowKey(item)}`,
    name: item.player_name,
    note: `${item.goals} 球`,
    avatar: item.avatar_storage_url,
    avatarMode: 'fill' as const,
    share: item.share,
    variant,
  }))
}

export function buildAssistContributionRows(
  scope: string,
  teamId: number | null,
  items: AssistContribution[],
): InsightContributionRow[] {
  return items.map(item => ({
    key: assistRowKey(item),
    instanceKey: `${teamId ?? 'none'}-${scope}-${assistRowKey(item)}`,
    name: item.player_name,
    note: `${item.assists} 次`,
    avatar: item.avatar_storage_url,
    avatarMode: 'fill' as const,
    share: item.share,
    variant: 'green' as const,
  }))
}
