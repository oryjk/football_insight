import type { TransferCard, TransferIntent, TransferPlayer, TransferTally } from '../types/transfer'

/** 转会投票页面路径与文案：洞察页、首页、分包页面共用，页面之间不互相引用。 */
export const TRANSFER_INTENTS: TransferIntent[] = ['out', 'stay', 'in']
// 榜单名用球迷的口吻（最想送走谁），不用功能名（离队榜）。
export const TRANSFER_BOARD_LABELS: Record<TransferIntent, string> = { out: '最想送走', stay: '最想留下', in: '最想买入' }
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
/** 链接参数 code=，或扫小程序码进入时的 scene（s=share&c=transfer&t=<码>，可能经过 URL 编码）。 */
export function parseTransferCode(query: Record<string, unknown> | undefined): string {
  if (validTransferCode(query?.code)) return query!.code as string
  if (typeof query?.scene !== 'string') return ''
  try {
    const scene = decodeURIComponent(query.scene)
    const values = Object.fromEntries(scene.split('&').map(pair => pair.split('=')))
    return values.s === 'share' && values.c === 'transfer' && validTransferCode(values.t) ? values.t : ''
  } catch { return '' }
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

/** 转会投票的资格提示：洞察页、完整榜单页、单卡页共用同一套判断与文案。 */
export const TRANSFER_HINTS = {
  notFan: '只能为你的主队投票，可在首页「我的主队」设置',
  guest: '登录并设置主队后可以投票',
  closed: '本期投票已结束',
} as const

interface TransferEligibility { loggedIn: boolean; canVote: boolean; open: boolean }

/** 点赞踩或提名时：已登录却不能投就给出原因；游客返回空（交给登录流程），能投返回空。 */
export function transferBlockedMessage(input: TransferEligibility): string {
  if (!input.loggedIn || input.canVote) return ''
  return input.open ? TRANSFER_HINTS.notFan : TRANSFER_HINTS.closed
}

/** 页面上常驻的提示：只在本期开放、且当前浏览者不能投时显示。 */
export function transferHint(input: TransferEligibility): string {
  if (!input.open || input.canVote) return ''
  return input.loggedIn ? TRANSFER_HINTS.notFan : TRANSFER_HINTS.guest
}
