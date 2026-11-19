import type { DirectorPlanDto, DirectorPosition } from '../types/director'
import { buildSharePath } from './shareSource'

export const DEFAULT_DIRECTOR_TEAM_ID = 77680

const POSITION_LABELS: Record<DirectorPosition, string> = {
  GK: '门将',
  DF: '后卫',
  MF: '中场',
  FW: '前锋',
}

const LEAGUE_LABELS: Record<string, string> = {
  K1: 'K1联赛',
  K2: 'K2联赛',
  K: 'K联赛',
  J1: 'J1联赛',
  EUR: '五大联赛',
  ACL: '亚冠',
  CSL: '中超',
  CL1: '中甲',
  SPL: '沙特联',
  MLS: '美职联',
  FREE: '自由身',
  BRA: '巴西',
  OTHER: '海外',
}

export function validDirectorCode(code: unknown): code is string {
  return typeof code === 'string' && /^[a-zA-Z0-9]{8}$/.test(code)
}

export interface DirectorEditLinkOptions {
  teamId?: number | null
  circleCode?: string | null
  fromCode?: string | null
}

export function buildDirectorEditUrl(options: DirectorEditLinkOptions = {}): string {
  const params: string[] = []
  if (typeof options.teamId === 'number' && Number.isInteger(options.teamId) && options.teamId > 0) {
    params.push(`team_id=${options.teamId}`)
  }
  if (validDirectorCode(options.circleCode)) {
    params.push(`circle_code=${options.circleCode}`)
  }
  if (validDirectorCode(options.fromCode)) {
    params.push(`from_code=${options.fromCode}`)
  }
  return params.length ? `/pages/director/index?${params.join('&')}` : '/pages/director/index'
}

export function parseDirectorEditQuery(query: Record<string, unknown> | undefined): {
  teamId: number
  circleCode: string
  fromCode: string
} {
  const rawTeam = query?.team_id ?? query?.teamId
  const parsedTeam = typeof rawTeam === 'string' || typeof rawTeam === 'number' ? Number(rawTeam) : 0
  const teamId = Number.isInteger(parsedTeam) && parsedTeam > 0 ? parsedTeam : 0
  const rawCircle = query?.circle_code
  const rawFrom = query?.from_code
  const circleCode = validDirectorCode(rawCircle) ? rawCircle : ''
  const fromCode = validDirectorCode(rawFrom) ? rawFrom : ''
  return { teamId, circleCode, fromCode }
}

export function buildDirectorViewUrl(code: string): string {
  return validDirectorCode(code) ? `/pages/director/view?code=${code}` : ''
}

export function parseDirectorViewCode(query: Record<string, unknown> | undefined): string {
  const rawCode = query?.code
  if (validDirectorCode(rawCode)) {
    return rawCode
  }
  const rawScene = query?.scene
  if (typeof rawScene !== 'string') {
    return ''
  }
  try {
    const scene = decodeURIComponent(rawScene)
    const entries = Object.fromEntries(scene.split('&').map(pair => pair.split('=')))
    return entries.s === 'share' && entries.c === 'director' && validDirectorCode(entries.t) ? entries.t : ''
  } catch {
    return ''
  }
}

export function buildDirectorSharePath(targetPath: string): string {
  return buildSharePath(targetPath, 'director')
}

export function directorEditShareTitle(teamName?: string): string {
  return teamName
    ? `我来当${teamName}体育总监！休赛期5外援你来怎么选？`
    : '我来当总监：休赛期建队模拟，快来PK你的5外援方案！'
}

export function directorPlanShareTitle(plan: Pick<DirectorPlanDto, 'team_name' | 'style_tag' | 'foreign_stays' | 'foreign_ins'>): string {
  const total = plan.foreign_stays.length + plan.foreign_ins.length
  return `我给${plan.team_name}定了${total}外援（${plan.style_tag}），不服来PK！`
}

export function directorPositionLabel(position: DirectorPosition | string): string {
  return POSITION_LABELS[position as DirectorPosition] ?? position
}

export function directorLeagueLabel(leagueTag: string): string {
  const key = leagueTag.trim().toUpperCase()
  return LEAGUE_LABELS[key] ?? (leagueTag.trim() || '海外')
}
