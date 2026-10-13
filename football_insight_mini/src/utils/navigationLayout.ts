export interface NavigationWindow {
  statusBarHeight?: number
  windowWidth?: number
}

export interface NavigationCapsule {
  top: number
  height: number
  width: number
}

export interface NavigationLayout {
  statusHeight: number
  barHeight: number
  rowOffset: number
  rowHeight: number
  totalHeight: number
  windowWidth: number
}

/** All platform coordinates are CSS pixels; rpx conversion happens only when rendering. */
export function calculateNavigationLayout(
  windowInfo: NavigationWindow,
  capsule?: NavigationCapsule,
): NavigationLayout {
  const statusHeight = Number.isFinite(windowInfo.statusBarHeight) && windowInfo.statusBarHeight! >= 0
    ? windowInfo.statusBarHeight! : 0
  const windowWidth = Number.isFinite(windowInfo.windowWidth) && windowInfo.windowWidth! > 0
    ? windowInfo.windowWidth! : 375
  const validCapsule = capsule
    && [capsule.top, capsule.height, capsule.width].every(Number.isFinite)
    && capsule.top >= statusHeight && capsule.height > 0 && capsule.width > 0
  const rowHeight = validCapsule ? capsule.height : 32
  const rowOffset = validCapsule ? capsule.top - statusHeight : 6
  const barHeight = validCapsule ? rowHeight + rowOffset * 2 : 44
  return { statusHeight, barHeight, rowOffset, rowHeight, totalHeight: statusHeight + barHeight, windowWidth }
}

export function navigationPageStyle(layout: NavigationLayout): string {
  const rpx = (pixels: number) => `${Number((pixels * 750 / layout.windowWidth).toFixed(6))}rpx`
  const properties: [string, number][] = [
    ['--fi-brand-nav-status-height', layout.statusHeight],
    ['--fi-brand-nav-bar-height', layout.barHeight],
    ['--fi-brand-nav-height', layout.totalHeight],
    ['--fi-brand-nav-row-offset', layout.rowOffset],
    ['--fi-space-brand-nav-row-size', layout.rowHeight],
  ]
  return properties.map(([name, value]) => `${name}:${rpx(value)};`).join('')
}
