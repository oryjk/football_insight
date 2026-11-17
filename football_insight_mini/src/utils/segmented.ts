/**
 * FiSegmented 滑块的内联样式：默认扣除轨道内边距和段间距，gap 为 null 时按零间距计算。
 * 位移使用百分比，所以任意段数都适用，不再写死 2 段或 3 段。
 * 选中项不在列表里时隐藏滑块，避免指向错误的段。
 */
export function segmentedThumbStyle(count: number, index: number, gap: string | null = 'var(--fi-space-6)'): string {
  if (!Number.isInteger(count) || count < 1) return ''
  const width = gap === null
    ? `width: calc(100% / ${count});`
    : `width: calc((100% - ${gap} * ${count + 1}) / ${count});`
  if (!Number.isInteger(index) || index < 0 || index >= count) {
    return `${width} opacity: 0;`
  }
  const transform = gap === null
    ? `transform: translateX(${index * 100}%);`
    : `transform: translateX(calc(${index * 100}% + ${index} * ${gap}));`
  return `${width} ${transform}`
}
