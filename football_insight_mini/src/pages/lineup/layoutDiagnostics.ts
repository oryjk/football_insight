import type { ComponentInternalInstance } from 'vue'
import type { Px } from './dragGeometry'
import { getDevBuildInfo } from '../../utils/devBuildInfo'

type Rect = { left: number; top: number; width: number; height: number }
type Expected = Array<Px & { slot: number }>
const rounded = (value: number) => Math.round(value * 10) / 10

// Only geometry is collected: no player identities, account data, or storage.
export function describePitchLayout(expected: Expected, area: Rect | null, movers: Rect[]) {
  const actual = movers.map((rect) => ({
    x: area ? rounded(rect.left - area.left) : null,
    y: area ? rounded(rect.top - area.top) : null,
    width: rect.width, height: rect.height,
  }))
  return {
    area,
    expected: expected.map(({ slot, x, y }) => ({ slot, x: rounded(x), y: rounded(y) })),
    expectedFinite: expected.every(({ x, y }) => Number.isFinite(x) && Number.isFinite(y)),
    expectedDistinct: new Set(expected.map(({ x, y }) => `${rounded(x)}:${rounded(y)}`)).size,
    actualDistinct: new Set(actual.map(({ x, y }) => `${x}:${y}`)).size,
    actual,
  }
}

export function schedulePitchLayoutDiagnostic(
  instance: ComponentInternalInstance,
  read: () => { expected: Expected; field: { width: number; height: number }; windowWidth: number },
): () => void {
  if (!import.meta.env.DEV) return () => {}
  let cancelled = false
  // Let native x/y animation settle before observing; this never changes layout.
  const timer = setTimeout(() => {
    if (cancelled) return
    const snapshot = read()
    const query = uni.createSelectorQuery().in(instance)
    query.select('.pitch__area').boundingClientRect()
    query.selectAll('.pitch__mover').boundingClientRect()
    query.exec((results) => {
      if (cancelled) return
      console.info('[lineup-layout]', JSON.stringify({
        build: getDevBuildInfo(), field: snapshot.field, windowWidth: snapshot.windowWidth,
        ...describePitchLayout(snapshot.expected, results?.[0] ?? null, results?.[1] ?? []),
      }))
    })
  }, 1500)
  return () => { cancelled = true; clearTimeout(timer) }
}
