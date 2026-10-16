export interface FiSegmentedItem {
  key: string
  label: string
  /** 未选中时的图标（深灰）。 */
  icon?: string
  /** 选中时的图标（白色）；缺省沿用 icon。 */
  iconActive?: string
}
