import type { DirectorPosition } from '../../types/director'

export type PesPositionZone = 'gk' | 'df' | 'mf' | 'fw'

export interface PesPositionBadge {
  code: string
  zone: PesPositionZone
}

export type CandidateTagTone =
  | 'gold'
  | 'crimson'
  | 'orange'
  | 'emerald'
  | 'cyan'
  | 'indigo'
  | 'blue'
  | 'rose'
  | 'teal'
  | 'purple'

const ALL_TAG_TONES: CandidateTagTone[] = [
  'gold',
  'crimson',
  'orange',
  'emerald',
  'cyan',
  'indigo',
  'blue',
  'rose',
  'teal',
  'purple',
]

export function resolvePesPositionBadge(
  position: DirectorPosition | string,
  tags: readonly string[] = [],
): PesPositionBadge {
  const joined = tags.join(' ')
  const pos = position.trim().toUpperCase()

  if (pos === 'GK') {
    return { code: 'GK', zone: 'gk' }
  }
  if (pos === 'DF') {
    if (/左后卫|左路/.test(joined)) return { code: 'LB', zone: 'df' }
    if (/右后卫|边卫|翼卫/.test(joined)) return { code: 'RB', zone: 'df' }
    return { code: 'CB', zone: 'df' }
  }
  if (pos === 'MF') {
    if (/后腰|铁闸|拦截/.test(joined)) return { code: 'DMF', zone: 'mf' }
    if (/前腰|组织|大脑|节拍器/.test(joined)) return { code: 'AMF', zone: 'mf' }
    if (/边锋|边路/.test(joined)) return { code: 'RMF', zone: 'mf' }
    return { code: 'CMF', zone: 'mf' }
  }
  if (/边锋|内切|边路/.test(joined)) return { code: 'LWF', zone: 'fw' }
  if (/影锋|传射/.test(joined)) return { code: 'SS', zone: 'fw' }
  return { code: 'CF', zone: 'fw' }
}

const TAG_TONE_RULES: Array<{ tone: CandidateTagTone; pattern: RegExp }> = [
  { tone: 'purple', pattern: /U23|新星|潜力/ },
  { tone: 'gold', pattern: /金靴|助攻王|最佳阵|冠军|名将|流量|核心|大腿|功臣|大师/ },
  { tone: 'indigo', pattern: /英超|西甲|意甲|德甲|法甲|欧冠|欧战|留洋|沙特联|美职联|海外/ },
  { tone: 'blue', pattern: /K1|J1|亚冠/ },
  { tone: 'rose', pattern: /中超|中甲|国脚/ },
  { tone: 'teal', pattern: /免签|性价比|淘宝|即战力|经验/ },
  { tone: 'cyan', pattern: /中卫|防线|防空|防守|门将|国门|门线|反应|强壮/ },
  { tone: 'emerald', pattern: /中场|B2B|铁闸|前腰|组织|调度|传中|传射|拦截|后腰/ },
  { tone: 'orange', pattern: /边路|边锋|快马|反击|爆点|爆破|突击|突破|冲击力|推进|往返|边卫|边中|能中能边|跑动/ },
  { tone: 'crimson', pattern: /中锋|射手|神锋|强锋|支点|轰炸|头球|杀手|终结|门前|抢点|背身|内切|重炮|远射|替补|机会/ },
]

export function resolveCandidateTagTone(tag: string): CandidateTagTone {
  const normalized = tag.trim()
  for (const rule of TAG_TONE_RULES) {
    if (rule.pattern.test(normalized)) {
      return rule.tone
    }
  }
  let hash = 0
  for (let i = 0; i < normalized.length; i += 1) {
    hash = (hash * 31 + normalized.charCodeAt(i)) >>> 0
  }
  return ALL_TAG_TONES[hash % ALL_TAG_TONES.length]
}
