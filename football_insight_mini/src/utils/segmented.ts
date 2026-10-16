/**
 * FiSegmented 滑块的内联样式：宽度按段数均分（扣掉轨道两侧内边距和段间距），
 * 位移用百分比（相对滑块自身宽度）加间距累计，所以任意段数都适用，不再写死 2 段或 3 段。
 * 选中项不在列表里时隐藏滑块，避免指向错误的段。
 */
export function segmentedThumbStyle(count: number, index: number): string {
  if (!Number.isInteger(count) || count < 1) return ''
  const width = `width: calc((100% - var(--fi-space-6) * ${count + 1}) / ${count});`
  if (!Number.isInteger(index) || index < 0 || index >= count) {
    return `${width} opacity: 0;`
  }
  return `${width} transform: translateX(calc(${index * 100}% + ${index} * var(--fi-space-6)));`
}
