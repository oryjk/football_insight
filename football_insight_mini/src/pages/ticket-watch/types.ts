import type {
  TicketWatchMatchSummary,
  TicketWatchGroupedInventoryItem,
} from '../../types/ticketWatch'

export type TicketWatchBoardMode = 'current' | 'history'
export type TicketWatchTab = TicketWatchBoardMode | 'history-stats'
export type PendingInterestSelection = {
  match: TicketWatchMatchSummary
  item: TicketWatchGroupedInventoryItem
  mode: TicketWatchBoardMode
  blockName: string
}
export type TicketWatchTeam = 'chengdurongcheng' | 'yunnanyukun'
