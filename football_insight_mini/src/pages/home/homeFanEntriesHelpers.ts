import type { LineupConsensus } from '../../types/lineup'
import type { SupportProfile } from '../../types/support'
import type { TransferCard, TransferIntent, TransferSummary } from '../../types/transfer'
import { DEFAULT_FORMATION, getFormation } from '../../utils/lineupFormations'

export const DEFAULT_FAN_ENTRY_TEAM_ID = 77680
export const DEFAULT_FAN_ENTRY_TEAM_NAME = '成都蓉城'

export interface HomeFanTeamResolution {
  teamId: number
  teamName: string
  ready: boolean
}

export interface MiniPitchDot {
  slot: number
  style: string
}

export interface HomeLineupEntryViewModel {
  kicker: string
  title: string
  subtitle: string
  ctaLabel: string
  pitchDots: MiniPitchDot[]
}

export interface HomeTransferAvatar {
  playerId: number
  name: string
  avatarUrl: string | null
}

export interface HomeTransferEntryViewModel {
  kicker: string
  title: string
  subtitle: string
  ctaLabel: string
  likeBadgeText: string
  avatars: HomeTransferAvatar[]
}

export function resolveHomeFanTeam(options: {
  hasAuthToken: boolean
  supportLoading: boolean
  profile: SupportProfile | null
}): HomeFanTeamResolution {
  const favorite = options.profile?.favorite_team ?? null
  return {
    teamId: favorite?.team_id ?? DEFAULT_FAN_ENTRY_TEAM_ID,
    teamName: favorite?.team_name ?? DEFAULT_FAN_ENTRY_TEAM_NAME,
    ready: !options.hasAuthToken || options.profile !== null || !options.supportLoading,
  }
}

function clampPitchPct(value: number): number {
  return Number.isFinite(value) ? Math.min(90, Math.max(10, value * 100)) : 50
}

export function buildHomeLineupEntryViewModel(
  consensus: LineupConsensus | null,
  teamName: string,
): HomeLineupEntryViewModel {
  const formationCode = consensus?.unlocked && consensus.formation ? consensus.formation : DEFAULT_FORMATION
  const pitchDots = getFormation(formationCode).map((slot) => ({
    slot: slot.slot,
    style: `left:${clampPitchPct(slot.x)}%;top:${clampPitchPct(slot.y)}%`,
  }))

  let subtitle = `排一套${teamName}最佳 11 人`
  if (consensus?.unlocked && consensus.formation) {
    subtitle = `${consensus.total} 套阵容 · ${consensus.formation}`
  } else if (consensus && consensus.total > 0) {
    subtitle = `${consensus.total} 套阵容 · 汇聚中`
  }

  return {
    kicker: '最佳 11 人',
    title: '排阵容',
    subtitle,
    ctaLabel: '去排阵',
    pitchDots,
  }
}

const FEATURED_INTENT_ORDER: TransferIntent[] = ['stay', 'in', 'out']

export function buildHomeTransferEntryViewModel(
  summary: TransferSummary | null,
  teamName: string,
): HomeTransferEntryViewModel {
  const boardsByIntent = new Map((summary?.boards ?? []).map((board) => [board.intent, board]))
  const orderedCards: TransferCard[] = []
  for (const intent of FEATURED_INTENT_ORDER) {
    orderedCards.push(...(boardsByIntent.get(intent)?.items ?? []))
  }

  const avatars: HomeTransferAvatar[] = []
  const seenPlayerIds = new Set<number>()
  for (const card of orderedCards) {
    if (seenPlayerIds.has(card.player.player_id)) continue
    seenPlayerIds.add(card.player.player_id)
    avatars.push({
      playerId: card.player.player_id,
      name: card.player.name,
      avatarUrl: card.player.avatar_url,
    })
    if (avatars.length >= 3) break
  }

  const stayLead = boardsByIntent.get('stay')?.items[0] ?? null
  const bestOverallLead = [...orderedCards].sort((left, right) => right.up - left.up)[0] ?? null
  const leadCard = stayLead ?? bestOverallLead

  const totalNominations = (summary?.boards ?? []).reduce((sum, board) => sum + board.total, 0)
  const topLikesCount = leadCard?.up ?? 0

  const subtitle = leadCard && totalNominations > 0
    ? `${leadCard.player.name}等 ${totalNominations} 人热议中`
    : `留谁走谁 · 给${teamName}球员点赞`

  return {
    kicker: '球迷态度',
    title: '球员点赞',
    subtitle,
    ctaLabel: '去点赞',
    likeBadgeText: topLikesCount > 0 ? String(topLikesCount) : '热赞',
    avatars,
  }
}
