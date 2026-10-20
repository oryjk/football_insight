import type { TransferBoard, TransferCandidates, TransferCardView, TransferCodeImage, TransferIntent, TransferNominateResult, TransferSummary, TransferTally } from '../types/transfer'
import { request } from '../utils/request'

// auth: true 表示"有 token 就带上"：游客也能读，登录后多返回 my_vote / can_vote。
export function getTransferSummary(teamId: number): Promise<TransferSummary> {
  return request({ url: `/transfers/teams/${teamId}/summary`, auth: true })
}
export function getTransferBoard(teamId: number, intent: TransferIntent, page = 1): Promise<TransferBoard> {
  return request({ url: `/transfers/teams/${teamId}/board?intent=${intent}&page=${page}`, auth: true })
}
export function getTransferCard(code: string): Promise<TransferCardView> {
  return request({ url: `/transfers/nominations/${encodeURIComponent(code)}`, auth: true })
}
export function getTransferCandidates(teamId: number, intent: TransferIntent): Promise<TransferCandidates> {
  return request({ url: `/transfers/teams/${teamId}/candidates?intent=${intent}` })
}
export function nominateTransfer(teamId: number, playerId: number, intent: TransferIntent): Promise<TransferNominateResult> {
  return request({ url: `/transfers/teams/${teamId}/nominations`, method: 'POST', auth: true, data: { player_id: playerId, intent } })
}
/** 0 表示取消投票。 */
export function setTransferVote(code: string, value: 1 | -1 | 0): Promise<TransferTally> {
  const url = `/transfers/nominations/${encodeURIComponent(code)}/vote`
  return value === 0
    ? request({ url, method: 'DELETE', auth: true })
    : request({ url, method: 'PUT', auth: true, data: { value } })
}
export function getTransferCode(code: string): Promise<TransferCodeImage> {
  return request({ url: `/transfers/nominations/${encodeURIComponent(code)}/wxacode`, auth: true })
}
