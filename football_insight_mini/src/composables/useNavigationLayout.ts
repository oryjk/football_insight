import { computed, ref } from 'vue'
import { onLoad, onReady, onResize, onShow } from '@dcloudio/uni-app'
import {
  calculateNavigationLayout,
  navigationPageStyle as formatPageStyle,
  type NavigationLayout,
  type NavigationWindow,
} from '../utils/navigationLayout'

const layout = ref<NavigationLayout | null>(null)

/** Share viewport geometry, and refresh it when each real page becomes ready or visible. */
export function useNavigationLayout() {
  function syncLayout(): void {
    // #ifdef MP-WEIXIN
    let windowInfo: NavigationWindow
    try {
      windowInfo = uni.getWindowInfo ? uni.getWindowInfo() : uni.getSystemInfoSync()
    } catch {
      try { windowInfo = uni.getSystemInfoSync() } catch { return }
    }
    try {
      const capsule = uni.getMenuButtonBoundingClientRect?.()
      layout.value = calculateNavigationLayout(windowInfo, capsule)
    } catch {
      layout.value = calculateNavigationLayout(windowInfo)
    }
    // #endif
  }
  onLoad(syncLayout)
  onShow(syncLayout)
  onReady(syncLayout)
  onResize(syncLayout)
  return {
    navigationPageStyle: computed(() => layout.value ? formatPageStyle(layout.value) : ''),
    /** 品牌导航总高度（CSS px，H5 为 0），供 IntersectionObserver 扣除被导航遮挡的区域。 */
    navigationHeightPx: computed(() => layout.value?.totalHeight ?? 0),
  }
}
