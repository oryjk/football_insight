import { describe, expect, test } from 'bun:test'
import { resolveCandidateTagTone, resolvePesPositionBadge } from './badges'

describe('director PES position badges', () => {
  test('maps positions and tags to PES abbreviations and zones', () => {
    expect(resolvePesPositionBadge('GK', ['国门级别'])).toEqual({ code: 'GK', zone: 'gk' })
    expect(resolvePesPositionBadge('DF', ['亚外中卫'])).toEqual({ code: 'CB', zone: 'df' })
    expect(resolvePesPositionBadge('DF', ['左路往返'])).toEqual({ code: 'LB', zone: 'df' })
    expect(resolvePesPositionBadge('DF', ['攻击边卫'])).toEqual({ code: 'RB', zone: 'df' })
    expect(resolvePesPositionBadge('MF', ['免签后腰'])).toEqual({ code: 'DMF', zone: 'mf' })
    expect(resolvePesPositionBadge('MF', ['技术前腰'])).toEqual({ code: 'AMF', zone: 'mf' })
    expect(resolvePesPositionBadge('MF', ['全能B2B'])).toEqual({ code: 'CMF', zone: 'mf' })
    expect(resolvePesPositionBadge('FW', ['内切边锋'])).toEqual({ code: 'LWF', zone: 'fw' })
    expect(resolvePesPositionBadge('FW', ['传射兼备'])).toEqual({ code: 'SS', zone: 'fw' })
    expect(resolvePesPositionBadge('FW', ['高中锋'])).toEqual({ code: 'CF', zone: 'fw' })
  })
})

describe('director candidate tag tones', () => {
  test('assigns distinct semantic tones across tag categories', () => {
    expect(resolveCandidateTagTone('K联赛金靴')).toBe('gold')
    expect(resolveCandidateTagTone('高中锋')).toBe('crimson')
    expect(resolveCandidateTagTone('边路快马')).toBe('orange')
    expect(resolveCandidateTagTone('全能B2B')).toBe('emerald')
    expect(resolveCandidateTagTone('亚外中卫')).toBe('cyan')
    expect(resolveCandidateTagTone('英超现役')).toBe('indigo')
    expect(resolveCandidateTagTone('K1现役')).toBe('blue')
    expect(resolveCandidateTagTone('中超旧将')).toBe('rose')
    expect(resolveCandidateTagTone('免签即战力')).toBe('teal')
    expect(resolveCandidateTagTone('U23新星')).toBe('purple')
  })
})
