import type { LineupPosition } from '../types/lineup'

export interface FormationSlot {
  slot: number
  position: LineupPosition
  label: string
  x: number
  y: number
}

export const FORMATION_CODES = ['4-3-3', '4-4-2', '4-2-3-1', '3-5-2', '3-4-3', '5-3-2', '4-1-4-1'] as const
export const DEFAULT_FORMATION = '4-3-3'

type FormationCode = (typeof FORMATION_CODES)[number]

// 坐标约定：x 从左到右 0–1；y 从对方球门（0）到己方球门（1），门将固定在 (0.5, 0.9)。
// 每行是同一条线上的一组球员，xs 与 labels 一一对应（从左到右）。
interface Row {
  position: LineupPosition
  y: number
  xs: number[]
  labels: string[]
}

const GK: Row = { position: 'GK', y: 0.9, xs: [0.5], labels: ['门将'] }
const BACK_FOUR: Row = {
  position: 'DF',
  y: 0.74,
  xs: [0.12, 0.37, 0.63, 0.88],
  labels: ['左后卫', '左中卫', '右中卫', '右后卫'],
}
const BACK_THREE: Row = {
  position: 'DF',
  y: 0.74,
  xs: [0.25, 0.5, 0.75],
  labels: ['左中卫', '中卫', '右中卫'],
}

const ROWS: Record<FormationCode, Row[]> = {
  '4-3-3': [
    GK,
    BACK_FOUR,
    { position: 'MF', y: 0.52, xs: [0.25, 0.5, 0.75], labels: ['左中场', '中场', '右中场'] },
    { position: 'FW', y: 0.24, xs: [0.18, 0.82], labels: ['左边锋', '右边锋'] },
    { position: 'FW', y: 0.18, xs: [0.5], labels: ['中锋'] },
  ],
  '4-4-2': [
    GK,
    BACK_FOUR,
    { position: 'MF', y: 0.5, xs: [0.12, 0.37, 0.63, 0.88], labels: ['左中场', '左中前卫', '右中前卫', '右中场'] },
    { position: 'FW', y: 0.2, xs: [0.35, 0.65], labels: ['左前锋', '右前锋'] },
  ],
  '4-2-3-1': [
    GK,
    BACK_FOUR,
    { position: 'MF', y: 0.57, xs: [0.35, 0.65], labels: ['左后腰', '右后腰'] },
    { position: 'MF', y: 0.38, xs: [0.18, 0.5, 0.82], labels: ['左前腰', '前腰', '右前腰'] },
    { position: 'FW', y: 0.18, xs: [0.5], labels: ['前锋'] },
  ],
  '3-5-2': [
    GK,
    BACK_THREE,
    { position: 'MF', y: 0.56, xs: [0.3, 0.7], labels: ['左中场', '右中场'] },
    { position: 'MF', y: 0.52, xs: [0.08, 0.92], labels: ['左翼卫', '右翼卫'] },
    { position: 'MF', y: 0.44, xs: [0.5], labels: ['前腰'] },
    { position: 'FW', y: 0.2, xs: [0.35, 0.65], labels: ['左前锋', '右前锋'] },
  ],
  '3-4-3': [
    GK,
    BACK_THREE,
    { position: 'MF', y: 0.5, xs: [0.12, 0.37, 0.63, 0.88], labels: ['左中场', '左中前卫', '右中前卫', '右中场'] },
    { position: 'FW', y: 0.24, xs: [0.18, 0.82], labels: ['左边锋', '右边锋'] },
    { position: 'FW', y: 0.18, xs: [0.5], labels: ['中锋'] },
  ],
  '5-3-2': [
    GK,
    {
      position: 'DF',
      y: 0.72,
      xs: [0.06, 0.27, 0.5, 0.73, 0.94],
      labels: ['左翼卫', '左中卫', '中卫', '右中卫', '右翼卫'],
    },
    { position: 'MF', y: 0.5, xs: [0.25, 0.5, 0.75], labels: ['左中场', '中场', '右中场'] },
    { position: 'FW', y: 0.2, xs: [0.35, 0.65], labels: ['左前锋', '右前锋'] },
  ],
  '4-1-4-1': [
    GK,
    BACK_FOUR,
    { position: 'MF', y: 0.62, xs: [0.5], labels: ['后腰'] },
    { position: 'MF', y: 0.44, xs: [0.12, 0.37, 0.63, 0.88], labels: ['左中场', '左中前卫', '右中前卫', '右中场'] },
    { position: 'FW', y: 0.18, xs: [0.5], labels: ['前锋'] },
  ],
}

/** 阵型的 11 个位置：序号 0 为门将，其后依次后卫、中场、前锋，同类别内按配置顺序从左到右。未知代码回退默认阵型。 */
export function getFormation(code: string): FormationSlot[] {
  const known = (FORMATION_CODES as readonly string[]).includes(code)
  const rows = ROWS[(known ? code : DEFAULT_FORMATION) as FormationCode]
  const result: FormationSlot[] = []
  let slot = 0
  for (const row of rows) {
    row.xs.forEach((x, index) => {
      result.push({ slot, position: row.position, label: row.labels[index], x, y: row.y })
      slot += 1
    })
  }
  return result
}
