import type { TicketWatchTeam } from './types'

interface BoardRequestOwner {
  owner: number
  revision: number
  team: TicketWatchTeam
}

export interface TicketWatchBoardRequests {
  historySelectionRequestToken: number
  currentBoardRequestSeq: number
  activeCurrentBoardRequest: BoardRequestOwner | null
  historyBoardRequestSeq: number
  activeHistoryBoardRequest: BoardRequestOwner | null
  historyTrendRequestSeq: number
}

// 看板请求 owner：reset/球队切换作废旧请求并允许新上下文立即重载；
// 所有 success/catch/finally 都核对 owner，旧请求不写数据、不清新 loading。
// owner 与订阅 generation（currentBoardGeneration）独立：合法 board 请求自己的
// publishCurrentMatch 会递增订阅 generation，不能以 generation 判定自身新旧。
// 每次 useTicketWatchBoards 调用创建一份，当前/历史模块共享同一实例。
export function createTicketWatchBoardRequests(): TicketWatchBoardRequests {
  return {
    historySelectionRequestToken: 0,
    currentBoardRequestSeq: 0,
    activeCurrentBoardRequest: null,
    historyBoardRequestSeq: 0,
    activeHistoryBoardRequest: null,
    historyTrendRequestSeq: 0,
  }
}
