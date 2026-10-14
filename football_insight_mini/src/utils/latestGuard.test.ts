import { describe, expect, test } from 'bun:test'
import { createLatestGuard } from './latestGuard'

describe('createLatestGuard', () => {
  test('只有最新一次 begin 的结果仍有效', () => {
    const guard = createLatestGuard()
    const first = guard.begin()
    const second = guard.begin()

    expect(first.isCurrent()).toBe(false)
    expect(first.owns()).toBe(false)
    expect(second.isCurrent()).toBe(true)
    expect(second.owns()).toBe(true)
  })

  test('invalidate 让在途操作全部失效', () => {
    const guard = createLatestGuard()
    const token = guard.begin()

    guard.invalidate()

    expect(token.isCurrent()).toBe(false)
    expect(token.owns()).toBe(false)
  })

  test('上下文快照变化后结果失效，但仍由该操作负责收尾', () => {
    let revision = 1
    let matchId = 10
    const guard = createLatestGuard(() => [revision, matchId])
    const token = guard.begin()

    revision = 2
    expect(token.isCurrent()).toBe(false)
    expect(token.owns()).toBe(true)

    revision = 1
    matchId = 11
    expect(token.isCurrent()).toBe(false)

    matchId = 10
    expect(token.isCurrent()).toBe(true)
  })

  test('多个 guard 彼此独立', () => {
    const load = createLatestGuard()
    const vote = createLatestGuard()
    const voteToken = vote.begin()

    load.begin()
    load.invalidate()

    expect(voteToken.isCurrent()).toBe(true)
  })
})
