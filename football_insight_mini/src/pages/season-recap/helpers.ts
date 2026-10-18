import { parseAcquisitionLaunch } from '../../utils/acquisitionSource'
export function parseHomeDistance(value: string, required: boolean): number | null {
 const input = value.trim()
 if (!input && !required) return null
 if (!input) throw Error('请填写去主场的单边距离')
 if (!/^\d+(?:\.\d{1,2})?$/.test(input) || Number(input) > 500) throw Error('请输入 0–500 公里的数字，最多两位小数')
 return Number(input)
}
export function recapSharePath(season: number): string { return `/pages/season-recap/index?season=${season}&src=share&cid=recap${season}` }
export function resolveRecapSeason(query: Record<string, unknown>): number {
 const source = parseAcquisitionLaunch({query})
 const candidate = typeof query.season === 'string' ? query.season : source?.src === 'share' ? /^recap(\d{4})$/.exec(source.cid ?? '')?.[1] : undefined
 const year = Number(candidate)
 return Number.isInteger(year) && year >= 2026 && year <= new Date().getFullYear() ? year : 2026
}
