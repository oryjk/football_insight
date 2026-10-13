import { expect, test } from 'bun:test'
import { createTopicAudioClock } from './topicAudioClock'
test('暂停后跳转立即显示目标时间，旧回调不得回退到0:01', () => {
  const clock = createTopicAudioClock()
  clock.observe(1)
  expect(clock.seek(45, 120)).toBe(45)
  expect(clock.observe(1)).toBe(45)
  expect(clock.observe(1.2)).toBe(45)
  expect(clock.observe(45)).toBe(45)
  expect(clock.observe(46)).toBe(46)
})
test('播放中向前向后跳转，延迟到达的新时间仍能继续推进', () => {
  let now = 0
  const clock = createTopicAudioClock(() => now)
  clock.setPlaying(true)
  clock.observe(20)
  clock.seek(70, 120)
  expect(clock.observe(21)).toBe(70)
  now = 3000
  expect(clock.observe(73)).toBe(73)
  clock.seek(10, 120)
  expect(clock.observe(74)).toBe(10)
  now = 5000
  expect(clock.observe(12)).toBe(12)
  expect(clock.observe(13)).toBe(13)
})
test('跳转边界、无效值及关闭重置', () => {
  const clock = createTopicAudioClock()
  expect(clock.seek(-3, 100)).toBe(0)
  expect(clock.seek(150, 100)).toBe(100)
  expect(clock.seek(NaN, 100)).toBeNull()
  expect(clock.seek(10, 0)).toBeNull()
  clock.reset()
  expect(clock.observe(1)).toBe(1)
})

test('连续拖拽时旧跳转的回调不能覆盖最后一次目标', () => {
  const clock = createTopicAudioClock()
  clock.observe(80)
  clock.seek(10, 120)
  clock.seek(40, 120)
  expect(clock.observe(80)).toBe(40)
  expect(clock.observe(10)).toBe(40)
  expect(clock.observe(40)).toBe(40)
})

test('暂停或缓冲期间时间流逝不能让旧0:01回调重新生效', () => {
  let now = 0
  const clock = createTopicAudioClock(() => now)
  clock.observe(1)
  clock.seek(0, 120)
  now = 600
  expect(clock.observe(1)).toBe(0)
  now = 100000
  expect(clock.observe(1)).toBe(0)
  expect(clock.observe(0)).toBe(0)
})
