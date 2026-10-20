/** 单次数字动画的帧数上限：约 32ms 一帧时总时长约 512ms，控制 setData 频率。 */
export const MAX_INTEGER_ANIMATION_FRAMES = 16

function easeOutProgress(ratio: number): number {
  return 1 - (1 - ratio) ** 3
}

/**
 * 按帧数生成 ease-out（先快后慢）的整数帧序列：
 * 首帧为 round(start)，末帧精确等于 round(target)，首帧之后单调趋近、不回弹也不越过目标；
 * 帧数不超过 frames 参数与 MAX_INTEGER_ANIMATION_FRAMES，也不超过 |目标-起点|+1（小差值不拖帧）。
 * 输入非负时输出恒非负。
 */
export function buildIntegerSequence(start: number, target: number, frames = MAX_INTEGER_ANIMATION_FRAMES): number[] {
  const from = Math.round(start)
  const to = Math.round(target)

  if (from === to) {
    return [to]
  }

  const frameBudget = Math.min(Math.max(1, Math.round(frames)), MAX_INTEGER_ANIMATION_FRAMES)
  const delta = to - from
  const stepCount = Math.max(1, Math.min(frameBudget - 1, Math.abs(delta)))
  const values: number[] = [from]

  for (let step = 1; step <= stepCount; step += 1) {
    // ease-out 在区间内单调且严格落在 from 与 to 之间，round 只可能重复、不可能反向或越界。
    const current = Math.round(from + delta * easeOutProgress(step / stepCount))
    if (current !== values[values.length - 1]) {
      values.push(current)
    }
  }

  values[values.length - 1] = to
  return values
}
