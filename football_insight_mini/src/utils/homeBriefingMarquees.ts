import type { HomeBriefingMarquees } from '../types/system'

export type HomeBriefingMarqueeAccent = 'leader' | 'scorer' | 'assist'

type HomeBriefingMarqueeMap = Record<HomeBriefingMarqueeAccent, string[]>

const EMPTY_ROWS: HomeBriefingMarqueeMap = {
  leader: [],
  scorer: [],
  assist: [],
}

function normalizeMessages(messages: string[] | undefined): string[] {
  if (!messages?.length) {
    return []
  }

  return messages
    .map((message) => message.trim())
    .filter((message) => message.length > 0)
}

export function buildHomeBriefingMarqueeMap(
  config: HomeBriefingMarquees | null | undefined,
): HomeBriefingMarqueeMap {
  if (!config) {
    return EMPTY_ROWS
  }

  return {
    leader: normalizeMessages(config.leader),
    scorer: normalizeMessages(config.scorer),
    assist: normalizeMessages(config.assist),
  }
}

export function splitBriefingMarqueeRows(messages: string[]): string[][] {
  if (!messages.length) {
    return []
  }

  const rowCount = Math.min(3, messages.length)
  const rows = Array.from({ length: rowCount }, () => [] as string[])

  messages.forEach((message, index) => {
    rows[index % rowCount].push(message)
  })

  return rows.filter((row) => row.length > 0)
}
