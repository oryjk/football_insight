import { nextTick } from 'vue'
import { tabAnchorScrollTop } from '../utils/tabAnchor'

/** 切换 tab 后把页面无动画地定位到胶囊栏（仅当它已被滚到导航后面时）。selector 为页面内的 id 选择器。 */
export function anchorTabsToTop(selector: string, navHeight: () => number): void {
  void nextTick(() => {
    let rectTop = Number.NaN
    let scrollTop = Number.NaN
    uni.createSelectorQuery()
      .select(selector)
      .boundingClientRect((rect) => {
        const box = Array.isArray(rect) ? rect[0] : rect
        if (box && typeof box.top === 'number') rectTop = box.top
      })
      .selectViewport()
      .scrollOffset((offset) => {
        const info = Array.isArray(offset) ? offset[0] : offset
        if (info && typeof info.scrollTop === 'number') scrollTop = info.scrollTop
      })
      .exec(() => {
        const target = tabAnchorScrollTop({ rectTop, scrollTop, navHeight: navHeight() })
        if (target !== null) uni.pageScrollTo({ scrollTop: target, duration: 0 })
      })
  })
}
