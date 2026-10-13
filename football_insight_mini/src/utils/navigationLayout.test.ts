import { describe, expect, test } from 'bun:test'
import { calculateNavigationLayout, navigationPageStyle } from './navigationLayout'

describe('custom navigation uses the native capsule geometry', () => {
  test('Android status bar, capsule and content offset share one measured height', () => {
    const layout = calculateNavigationLayout(
      { statusBarHeight: 32, windowWidth: 393 }, { top: 40, height: 32, width: 87 },
    )
    expect(layout.barHeight).toBe(48)
    expect(layout.totalHeight).toBe(80)
    expect(layout.statusHeight + layout.rowOffset + layout.rowHeight / 2).toBe(56)
    expect(layout.totalHeight - (40 + 32)).toBe(layout.rowOffset)
  })
  test('iPhone safe area does not use the fixed 25px uni-app CSS status bar', () => {
    const layout = calculateNavigationLayout(
      { statusBarHeight: 59, windowWidth: 390 }, { top: 65, height: 32, width: 87 },
    )
    expect(layout.barHeight).toBe(44)
    expect(layout.totalHeight).toBe(103)
    expect(layout.statusHeight + layout.rowOffset).toBe(65)
  })
  test('unavailable or invalid capsule falls back to 44px while retaining the real status bar', () => {
    for (const capsule of [undefined, { top: 0, height: 0, width: 0 }, { top: 20, height: 32, width: 87 }]) {
      const layout = calculateNavigationLayout({ statusBarHeight: 32, windowWidth: 375 }, capsule)
      expect(layout.totalHeight).toBe(76)
      expect(layout.rowOffset).toBe(6)
      expect(layout.rowHeight).toBe(32)
    }
  })
  test('rpx styles preserve measured pixels on wider screens', () => {
    const layout = calculateNavigationLayout(
      { statusBarHeight: 32, windowWidth: 414 }, { top: 40, height: 32, width: 87 },
    )
    const style = navigationPageStyle(layout)
    const properties = Object.fromEntries(style.split(';').filter(Boolean).map(item => item.split(':')))
    const pixels = (name: string) => parseFloat(properties[name]) * 414 / 750
    expect(Math.abs(pixels('--fi-brand-nav-height') - 80) < 0.00001).toBe(true)
    expect(Math.abs(pixels('--fi-brand-nav-row-offset') - 8) < 0.00001).toBe(true)
    expect(Math.abs(pixels('--fi-space-brand-nav-row-size') - 32) < 0.00001).toBe(true)
  })
})
