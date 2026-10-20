/**
 * 切换页内 tab 时的滚动定位：如果胶囊栏已经被滚到导航后面，返回让它刚好贴着导航的 scrollTop；
 * 胶囊栏仍在导航下方可见时返回 null（不动页面）。避免切到较短内容时页面被迫回弹造成的抖动。
 */
export function tabAnchorScrollTop(input: { rectTop: number; scrollTop: number; navHeight: number }): number | null {
  const { rectTop, scrollTop, navHeight } = input
  if (![rectTop, scrollTop, navHeight].every(Number.isFinite)) return null
  if (rectTop >= navHeight) return null
  return Math.max(0, scrollTop + rectTop - navHeight)
}
