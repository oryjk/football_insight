import { expect, test } from 'bun:test'
import { tabAnchorScrollTop } from './tabAnchor'

test('scrolled past the tab bar: pin the tab bar right under the navigation', () => {
  // 胶囊栏已经被滚到导航后面（top 比导航高度小），把页面定位到胶囊栏刚好贴着导航
  expect(tabAnchorScrollTop({ rectTop: -300, scrollTop: 900, navHeight: 88 })).toBe(512)
  expect(tabAnchorScrollTop({ rectTop: 40, scrollTop: 600, navHeight: 88 })).toBe(552)
})

test('tab bar still visible below the navigation: leave the page where it is', () => {
  expect(tabAnchorScrollTop({ rectTop: 88, scrollTop: 400, navHeight: 88 })).toBe(null)
  expect(tabAnchorScrollTop({ rectTop: 300, scrollTop: 0, navHeight: 88 })).toBe(null)
})

test('never scrolls to a negative position and ignores broken measurements', () => {
  expect(tabAnchorScrollTop({ rectTop: -500, scrollTop: 100, navHeight: 88 })).toBe(0)
  expect(tabAnchorScrollTop({ rectTop: Number.NaN, scrollTop: 100, navHeight: 88 })).toBe(null)
})
