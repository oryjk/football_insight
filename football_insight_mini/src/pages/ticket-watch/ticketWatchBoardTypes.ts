export type TicketWatchBoardApi = Pick<
  typeof import('../../api/ticketWatch'),
  | 'getCurrentTicketWatchBoard'
  | 'getTicketWatchBlockInterests'
  | 'getTicketWatchInventorySince'
  | 'getTicketWatchMatches'
  | 'getTicketWatchRegions'
  | 'getTicketWatchTrackedInterests'
  | 'getYukunTicketWatchMatches'
  | 'getYukunCurrentTicketWatchMatch'
  | 'getYukunTicketWatchInventory'
  | 'getYukunTicketWatchRegions'
>
