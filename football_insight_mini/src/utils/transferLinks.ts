import type { TransferCard, TransferIntent, TransferPlayer, TransferTally } from '../types/transfer'

/** 转会投票页面路径与文案：洞察页、首页、分包页面共用，页面之间不互相引用。 */
export const TRANSFER_INTENTS: TransferIntent[] = ['out', 'stay', 'in']
export const TRANSFER_BOARD_LABELS: Record<TransferIntent, string> = { out: '离队榜', stay: '留队榜', in: '引援榜' }
const SHARE_VERBS: Record<TransferIntent, string> = { out: '送走', stay: '留下', in: '买入' }
const POSITION_LABELS: Record<NonNullable<TransferPlayer['position']>, string> = { GK: '门将', DF: '后卫', MF: '中场', FW: '前锋' }

export function validTransferCode(code: unknown): code is string {
  return typeof code === 'string' && /^[a-zA-Z0-9]{8}$/.test(code)
}
function parseIntent(value: unknown): TransferIntent {
  return value === 'stay' || value === 'in' ? value : 'out'
}

export function buildTransferBoardUrl(teamId: number, intent: TransferIntent = 'out', options: { nominate?: boolean } = {}): string {
  if (!Number.isInteger(teamId) || teamId <= 0) return '/pages/transfer/board'
  return `/pages/transfer/board?teamId=${teamId}&intent=${intent}${options.nominate ? '&nominate=1' : ''}`
}
export function parseTransferBoardQuery(query: Record<string, unknown> | undefined): { teamId: number; intent: TransferIntent; nominate: boolean } {
  const raw = typeof query?.teamId === 'string' ? Number(query.teamId) : 0
  return { teamId: Number.isInteger(raw) && raw > 0 ? raw : 0, intent: parseIntent(query?.intent), nominate: query?.nominate === '1' }
}
export function buildTransferViewUrl(code: string): string {
  return validTransferCode(code) ? `/pages/transfer/view?code=${code}` : ''
}
export function parseTransferCode(query: Record<string, unknown> | undefined): string {
  return validTransferCode(query?.code) ? (query!.code as string) : ''
}

/** "我希望 X ___"里的动词。 */
export function transferStance(card: Pick<TransferCard, 'intent' | 'returning'>): string {
  if (card.intent === 'out') return '离队'
  if (card.intent === 'stay') return '留下'
  return card.returning ? '回归' : '加盟'
}
export function transferBoardShareTitle(teamName: string, windowName: string, intent: TransferIntent): string {
  return `${teamName}球迷最想${SHARE_VERBS[intent]}谁？${windowName}转会榜`
}
export function transferCardShareTitle(card: TransferCard): string {
  return `我希望 ${card.player.name} ${transferStance(card)}，你同意吗？`
}
export function applyTally(card: TransferCard, tally: TransferTally): TransferCard {
  return { ...card, up: tally.up, down: tally.down, my_vote: tally.my_vote }
}
export function positionLabel(position: TransferPlayer['position']): string {
  return position ? POSITION_LABELS[position] : ''
}
