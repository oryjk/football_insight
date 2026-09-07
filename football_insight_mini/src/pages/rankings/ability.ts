import type { PlayerRankingCategory, TeamRankingCategory } from '../../types/insight'

// 实况式六维能力：进攻、射门、组织、传球、防守、对抗。
// 各项按类目内最大值归一化，防守 = 拦截 + 解围，对抗 = 被侵犯（反映前场制造的威胁）。
export const TEAM_ABILITY_AXES = ['进攻', '射门', '组织', '传球', '防守', '对抗'] as const

const TEAM_ABILITY_CATEGORY_SLUGS = ['goals', 'shots_on_target', 'assists', 'pass_success_rate'] as const

function parseScoreValue(value: string | null | undefined): number {
  const parsed = Number.parseFloat(value ?? '')
  return Number.isFinite(parsed) ? parsed : 0
}

function buildCategoryScoreMap(
  teamCategories: TeamRankingCategory[],
  slug: string,
): Map<number, number> {
  const category = teamCategories.find((item) => item.slug === slug)
  const map = new Map<number, number>()
  for (const entry of category?.entries ?? []) {
    map.set(entry.team_id, parseScoreValue(entry.score_value))
  }
  return map
}

function normalizeScoreMap(map: Map<number, number>): Map<number, number> {
  const max = Math.max(0, ...map.values())
  if (max <= 0) {
    return new Map()
  }

  const normalized = new Map<number, number>()
  for (const [teamId, value] of map) {
    normalized.set(teamId, value / max)
  }
  return normalized
}

export function buildTeamAbilityValues(
  teamCategories: TeamRankingCategory[],
): Map<number, number[]> {
  const [goals, shotsOnTarget, assists, passSuccessRate] = TEAM_ABILITY_CATEGORY_SLUGS.map((slug) =>
    normalizeScoreMap(buildCategoryScoreMap(teamCategories, slug)),
  )

  const interceptions = buildCategoryScoreMap(teamCategories, 'interceptions')
  const clearances = buildCategoryScoreMap(teamCategories, 'clearances')
  const defenseRaw = new Map<number, number>()
  const defenseTeamIds = new Set([...interceptions.keys(), ...clearances.keys()])
  for (const teamId of defenseTeamIds) {
    defenseRaw.set(teamId, (interceptions.get(teamId) ?? 0) + (clearances.get(teamId) ?? 0))
  }
  const defense = normalizeScoreMap(defenseRaw)
  const duel = normalizeScoreMap(buildCategoryScoreMap(teamCategories, 'fouled'))

  const dimensions = [goals, shotsOnTarget, assists, passSuccessRate, defense, duel]
  const teamIds = new Set(dimensions.flatMap((map) => [...map.keys()]))

  const result = new Map<number, number[]>()
  for (const teamId of teamIds) {
    result.set(teamId, dimensions.map((map) => map.get(teamId) ?? 0))
  }
  return result
}

export const TEAM_ABILITY_FALLBACK = [0, 0, 0, 0, 0, 0]

// 球员六维与球队同构：进攻（进球）、射门（射正）、组织（助攻）、传球（传球）、
// 防守（拦截 + 解围）、对抗（被侵犯）。球员类目只有各类目前 20 名，未上榜维度按 0 计。
export const PLAYER_ABILITY_AXIS_UNITS = ['球', '次', '次', '脚', '次', '次'] as const

function buildPlayerCategoryScoreMap(
  playerCategories: PlayerRankingCategory[],
  slug: string,
): Map<number, number> {
  const category = playerCategories.find((item) => item.slug === slug)
  const map = new Map<number, number>()
  for (const entry of category?.entries ?? []) {
    map.set(entry.player_id, parseScoreValue(entry.score_value))
  }
  return map
}

function buildPlayerDimensionMaps(playerCategories: PlayerRankingCategory[]): {
  normalized: Map<number, number>[]
  raws: Map<number, number>[]
} {
  const [goals, shotsOnTarget, assists, passes] = ['goals', 'shots_on_target', 'assists', 'passes']
    .map((slug) => buildPlayerCategoryScoreMap(playerCategories, slug))

  const interceptions = buildPlayerCategoryScoreMap(playerCategories, 'interceptions')
  const clearances = buildPlayerCategoryScoreMap(playerCategories, 'clearances')
  const defenseRaw = new Map<number, number>()
  const defensePlayerIds = new Set([...interceptions.keys(), ...clearances.keys()])
  for (const playerId of defensePlayerIds) {
    defenseRaw.set(playerId, (interceptions.get(playerId) ?? 0) + (clearances.get(playerId) ?? 0))
  }
  const duel = buildPlayerCategoryScoreMap(playerCategories, 'fouled')

  const raws = [goals, shotsOnTarget, assists, passes, defenseRaw, duel]
  return { normalized: raws.map(normalizeScoreMap), raws }
}

export function buildPlayerAbilityValues(
  playerCategories: PlayerRankingCategory[],
): Map<number, number[]> {
  const { normalized } = buildPlayerDimensionMaps(playerCategories)
  const playerIds = new Set(normalized.flatMap((map) => [...map.keys()]))

  const result = new Map<number, number[]>()
  for (const playerId of playerIds) {
    result.set(playerId, normalized.map((map) => map.get(playerId) ?? 0))
  }
  return result
}

export interface PlayerAbilityDetail {
  values: number[]
  // 与 TEAM_ABILITY_AXES 顺序一致的原始数据展示文本，如 "19 球"
  rawTexts: string[]
}

export function buildPlayerAbilityDetail(
  playerCategories: PlayerRankingCategory[],
  playerId: number,
): PlayerAbilityDetail {
  const { normalized, raws } = buildPlayerDimensionMaps(playerCategories)

  return {
    values: normalized.map((map) => map.get(playerId) ?? 0),
    rawTexts: raws.map((map, index) => `${map.get(playerId) ?? 0} ${PLAYER_ABILITY_AXIS_UNITS[index]}`),
  }
}
