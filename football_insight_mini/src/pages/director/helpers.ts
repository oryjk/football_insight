import type {
  DirectorCandidateDto,
  DirectorLeagueFilter,
  DirectorPlanDto,
  DirectorPosition,
  PlanPlayerItem,
  UpsertDirectorPlayerInput,
} from '../../types/director'

export const DIRECTOR_DRAFT_STORAGE_KEY = 'fi-director-pending-draft-v1'
export const DIRECTOR_MANIFESTO_MAX = 40
export const DIRECTOR_COMMENT_MAX = 50

export const DIRECTOR_POSITIONS: ReadonlyArray<{ value: DirectorPosition; label: string }> = [
  { value: 'FW', label: '前锋' },
  { value: 'MF', label: '中场' },
  { value: 'DF', label: '后卫' },
  { value: 'GK', label: '门将' },
]

export const DIRECTOR_FOREIGN_LEAGUE_TABS: ReadonlyArray<{ value: DirectorLeagueFilter; label: string }> = [
  { value: 'all', label: '全部' },
  { value: 'K', label: 'K联赛' },
  { value: 'J1', label: 'J联赛' },
  { value: 'EUR', label: '五大联赛' },
  { value: 'ACL', label: '亚冠' },
  { value: 'CL1', label: '中甲' },
  { value: 'SPL', label: '沙特联' },
  { value: 'MLS', label: '美职联' },
  { value: 'CSL', label: '中超' },
  { value: 'FREE', label: '自由身' },
  { value: 'CUSTOM', label: '自由提名' },
]

export const DIRECTOR_DOMESTIC_LEAGUE_TABS: ReadonlyArray<{ value: DirectorLeagueFilter; label: string }> = [
  { value: 'all', label: '全部' },
  { value: 'CSL', label: '中超' },
  { value: 'CL1', label: '中甲' },
  { value: 'FREE', label: '自由身' },
  { value: 'CUSTOM', label: '自由提名' },
]

export const DIRECTOR_MANIFESTO_PRESETS: readonly string[] = [
  '把预算砸在K联赛淘宝，性价比拉满！',
  '稳字当头，微调冲亚冠！',
  '推倒重来，外援换血才有戏！',
  '中轴线全面升级，铁血防守赢天下！',
]

export interface StyleTagInput {
  foreignStaysCount: number
  foreignLeavesCount: number
  foreignIns: ReadonlyArray<Pick<PlanPlayerItem, 'league_tag' | 'position'>>
  domesticIns: ReadonlyArray<Pick<PlanPlayerItem, 'position'>>
}

export function computeDirectorStyleTag(input: StyleTagInput): string {
  const hasKLeagueSigning = input.foreignIns.some(item =>
    item.league_tag.toUpperCase().includes('K'),
  )
  if (hasKLeagueSigning) {
    if (input.foreignLeavesCount >= 3 && input.foreignIns.length >= 3) {
      return '推倒重建派'
    }
    return 'K联赛淘宝流'
  }
  if (input.foreignLeavesCount >= 3) {
    return '推倒重建派'
  }
  if (input.foreignStaysCount >= 4 && input.foreignIns.length <= 1) {
    return '稳健微调派'
  }

  const allIns = [...input.foreignIns, ...input.domesticIns]
  if (allIns.length >= 2) {
    const fwCount = allIns.filter(item => item.position === 'FW').length
    if (fwCount * 2 >= allIns.length) {
      return '锋线狂魔'
    }
    const defCount = allIns.filter(item => item.position === 'DF' || item.position === 'GK').length
    if (defCount * 2 >= allIns.length) {
      return '铁血防线派'
    }
  }
  return '务实均衡派'
}

export interface ForeignQuotaCheck {
  total: number
  quota: number
  remaining: number
  overLimit: boolean
  valid: boolean
  progressPercent: number
  message: string
}

export function validateForeignQuota(
  stayCount: number,
  inCount: number,
  quota = 5,
): ForeignQuotaCheck {
  const safeQuota = quota > 0 ? quota : 5
  const total = Math.max(0, stayCount) + Math.max(0, inCount)
  const remaining = Math.max(0, safeQuota - total)
  const overLimit = total > safeQuota
  const valid = total >= 1 && !overLimit
  const progressPercent = Math.min(100, Math.round((total / safeQuota) * 100))
  let message = ''
  if (total === 0) {
    message = '至少保留或签下 1 名外援'
  } else if (overLimit) {
    message = `外援名额超出上限（${total}/${safeQuota}），请先精简`
  }
  return { total, quota: safeQuota, remaining, overLimit, valid, progressPercent, message }
}

export function candidateToPlanPlayer(candidate: DirectorCandidateDto): PlanPlayerItem {
  return {
    key: candidate.key,
    source: candidate.source,
    player_id: candidate.player_id,
    candidate_code: candidate.candidate_code,
    player_name: candidate.player_name,
    position: candidate.position,
    country: candidate.country,
    is_foreign: candidate.is_foreign,
    league_tag: candidate.league_tag,
    current_club: candidate.current_club,
    avatar_url: candidate.avatar_url,
    market_value_eur: candidate.market_value_eur ?? null,
    birth_date: candidate.birth_date ?? null,
    contract_until: candidate.contract_until ?? null,
  }
}

export function createCustomPlanPlayer(input: {
  playerName: string
  position: DirectorPosition
  leagueTag: string
  currentClub: string
  isForeign: boolean
}): PlanPlayerItem {
  const name = input.playerName.trim()
  const league = input.leagueTag.trim() || (input.isForeign ? 'OTHER' : 'CSL')
  const club = input.currentClub.trim() || '自由提名'
  return {
    key: `custom:${name.toLowerCase()}`,
    source: 'custom',
    player_id: null,
    candidate_code: null,
    player_name: name,
    position: input.position,
    country: input.isForeign ? '海外' : '中国',
    is_foreign: input.isForeign,
    league_tag: league,
    current_club: club,
    avatar_url: null,
  }
}

export function toUpsertPlayerInput(item: PlanPlayerItem): UpsertDirectorPlayerInput {
  if (item.source === 'pool') {
    return {
      source: 'pool',
      candidate_code: item.candidate_code,
      player_id: null,
      player_name: null,
      position: null,
      league_tag: null,
      current_club: null,
    }
  }
  if (item.source === 'squad') {
    return {
      source: 'squad',
      candidate_code: null,
      player_id: item.player_id,
      player_name: null,
      position: null,
      league_tag: null,
      current_club: null,
    }
  }
  return {
    source: 'custom',
    candidate_code: null,
    player_id: null,
    player_name: item.player_name,
    position: item.position,
    league_tag: item.league_tag,
    current_club: item.current_club,
  }
}

export interface DirectorDraftSnapshot {
  teamId: number
  circleCode: string
  fromCode: string
  foreignStayIds: number[]
  foreignLeaveIds: number[]
  foreignIns: PlanPlayerItem[]
  domesticIns: PlanPlayerItem[]
  domesticOuts: number[]
  manifesto: string
  savedAt: number
}

export function serializeDirectorDraft(draft: DirectorDraftSnapshot): string {
  return JSON.stringify(draft)
}

export function deserializeDirectorDraft(raw: unknown): DirectorDraftSnapshot | null {
  if (typeof raw !== 'string' || !raw.trim()) {
    return null
  }
  try {
    const parsed = JSON.parse(raw) as Partial<DirectorDraftSnapshot>
    if (!parsed || typeof parsed !== 'object') {
      return null
    }
    if (typeof parsed.teamId !== 'number' || !Number.isInteger(parsed.teamId) || parsed.teamId <= 0) {
      return null
    }
    if (!Array.isArray(parsed.foreignStayIds) || !Array.isArray(parsed.foreignLeaveIds)) {
      return null
    }
    return {
      teamId: parsed.teamId,
      circleCode: typeof parsed.circleCode === 'string' ? parsed.circleCode : '',
      fromCode: typeof parsed.fromCode === 'string' ? parsed.fromCode : '',
      foreignStayIds: parsed.foreignStayIds.filter((id): id is number => typeof id === 'number'),
      foreignLeaveIds: parsed.foreignLeaveIds.filter((id): id is number => typeof id === 'number'),
      foreignIns: Array.isArray(parsed.foreignIns) ? parsed.foreignIns : [],
      domesticIns: Array.isArray(parsed.domesticIns) ? parsed.domesticIns : [],
      domesticOuts: Array.isArray(parsed.domesticOuts)
        ? parsed.domesticOuts.filter((id): id is number => typeof id === 'number')
        : [],
      manifesto: typeof parsed.manifesto === 'string' ? parsed.manifesto.slice(0, DIRECTOR_MANIFESTO_MAX) : '',
      savedAt: typeof parsed.savedAt === 'number' ? parsed.savedAt : Date.now(),
    }
  } catch {
    return null
  }
}

export function filterCandidatesByLeague(
  items: ReadonlyArray<DirectorCandidateDto>,
  league: DirectorLeagueFilter,
): DirectorCandidateDto[] {
  if (league === 'all' || league === 'CUSTOM') {
    return [...items]
  }
  if (league === 'K') {
    return items.filter(item => item.league_tag.toUpperCase().startsWith('K'))
  }
  return items.filter(item => item.league_tag.toUpperCase() === league)
}

export function buildDirectorShareText(
  plan: Pick<DirectorPlanDto, 'team_name' | 'style_tag' | 'foreign_stays' | 'foreign_ins' | 'manifesto'>,
): string {
  const totalForeign = plan.foreign_stays.length + plan.foreign_ins.length
  const prefix = `我给${plan.team_name}定了${totalForeign}外援（${plan.style_tag}）`
  const manifesto = plan.manifesto.trim()
  if (manifesto) {
    return `${prefix}：「${manifesto}」不服来PK！`
  }
  return `${prefix}，不服来微信群PK！`
}

export function countUnicodeChars(text: string): number {
  return Array.from(text).length
}

export function formatShortDateTime(iso: string): string {
  if (!iso) return ''
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return ''
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hour = String(date.getHours()).padStart(2, '0')
  const minute = String(date.getMinutes()).padStart(2, '0')
  return `${month}-${day} ${hour}:${minute}`
}

export function formatMarketValueEur(eur?: number | null): string {
  if (typeof eur !== 'number' || !Number.isFinite(eur) || eur <= 0) {
    return ''
  }
  if (eur >= 100_000_000) {
    const billions = (eur / 100_000_000).toFixed(2).replace(/\.?0+$/, '')
    return `${billions}亿欧`
  }
  if (eur >= 10_000) {
    const wan = eur / 10_000
    const formatted = wan >= 100 ? String(Math.round(wan)) : wan.toFixed(1).replace(/\.0$/, '')
    return `${formatted}万欧`
  }
  return `€${Math.round(eur)}`
}

export function computePlayerAge(birthDate?: string | null, today: Date = new Date()): number | null {
  if (!birthDate) return null
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(birthDate.trim())
  if (!match) return null
  const year = Number(match[1])
  const month = Number(match[2])
  const day = Number(match[3])
  if (!Number.isInteger(year) || month < 1 || month > 12 || day < 1 || day > 31) {
    return null
  }
  let age = today.getFullYear() - year
  const currentMonth = today.getMonth() + 1
  if (currentMonth < month || (currentMonth === month && today.getDate() < day)) {
    age -= 1
  }
  return age >= 12 && age <= 55 ? age : null
}

export function formatPlayerAge(birthDate?: string | null, today?: Date): string {
  const age = computePlayerAge(birthDate, today)
  return age === null ? '' : `${age}岁`
}

export function formatContractUntil(contractUntil?: string | null): string {
  if (!contractUntil) return ''
  const match = /^(\d{4})/.exec(contractUntil.trim())
  return match ? `合同至${match[1]}` : ''
}

export function sumMarketValueEur(
  items: ReadonlyArray<{ market_value_eur?: number | null }>,
): number {
  let total = 0
  for (const item of items) {
    if (typeof item.market_value_eur === 'number' && Number.isFinite(item.market_value_eur) && item.market_value_eur > 0) {
      total += item.market_value_eur
    }
  }
  return total
}
