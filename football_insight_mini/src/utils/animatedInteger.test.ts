import { expect, test } from 'bun:test'
import { buildIntegerSequence, MAX_INTEGER_ANIMATION_FRAMES } from './animatedInteger'

function isMonotonicTowards(sequence: number[], to: number): boolean {
  return sequence.every((value, index) => {
    if (index === 0) return true
    return to > sequence[0] ? value >= sequence[index - 1] : value <= sequence[index - 1]
  })
}

test('increasing sequences start at from, ease out and land exactly on the target', () => {
  const sequence = buildIntegerSequence(0, 3000)
  expect(sequence[0]).toBe(0)
  expect(sequence[sequence.length - 1]).toBe(3000)
  expect(isMonotonicTowards(sequence, 3000)).toBe(true)
  // ease-out：前半段覆盖的进度超过一半，且首步大于末步。
  const half = sequence[Math.floor(sequence.length / 2)]
  expect(half > 1500).toBe(true)
  expect(sequence[1] - sequence[0] > sequence[sequence.length - 1] - sequence[sequence.length - 2]).toBe(true)
})

test('decreasing sequences stay monotonic and end exactly on the target', () => {
  const sequence = buildIntegerSequence(2400, 0)
  expect(sequence[0]).toBe(2400)
  expect(sequence[sequence.length - 1]).toBe(0)
  expect(isMonotonicTowards(sequence, 0)).toBe(true)
})

test('frame count respects the requested budget and the global cap', () => {
  expect(buildIntegerSequence(0, 100000).length).toBe(MAX_INTEGER_ANIMATION_FRAMES)
  // frames 是上限预算：easing 相邻步取整重复时帧数可能更少，但至少有起止两帧。
  const budgeted = buildIntegerSequence(0, 100, 8)
  expect(budgeted.length <= 8).toBe(true)
  expect(budgeted.length > 1).toBe(true)
  expect(buildIntegerSequence(0, 100, 99).length <= MAX_INTEGER_ANIMATION_FRAMES).toBe(true)
  // 小差值不拖帧：帧数不超过差值 + 1。
  const small = buildIntegerSequence(3, 5)
  expect(small[0]).toBe(3)
  expect(small[small.length - 1]).toBe(5)
  expect(small.length <= 3).toBe(true)
  expect(buildIntegerSequence(5, 4)).toEqual([5, 4])
})

test('equal values collapse to a single frame', () => {
  expect(buildIntegerSequence(5, 5)).toEqual([5])
  expect(buildIntegerSequence(4.6, 5.4)).toEqual([5])
})

test('non-negative inputs never produce negative frames', () => {
  for (const sequence of [buildIntegerSequence(2, 4000), buildIntegerSequence(40, 0), buildIntegerSequence(0, 7)]) {
    expect(sequence.every(v => v >= 0)).toBe(true)
  }
})

test('fractional inputs are rounded and the final frame is exact for awkward deltas', () => {
  for (const [from, to] of [[7.4, 100.2], [98, 3.2], [0.3, 12.9]] as const) {
    const sequence = buildIntegerSequence(from, to)
    expect(sequence[0]).toBe(Math.round(from))
    expect(sequence[sequence.length - 1]).toBe(Math.round(to))
    expect(isMonotonicTowards(sequence, Math.round(to))).toBe(true)
  }
})
