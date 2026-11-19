import { describe, expect, test } from 'bun:test'
import {
  allocateBackInterceptId,
  isTopBackIntercept,
  pushBackIntercept,
  removeBackIntercept,
} from './backIntercept'

describe('backIntercept stack', () => {
  test('tracks topmost active intercept across stacked sheets', () => {
    const first = allocateBackInterceptId()
    const second = allocateBackInterceptId()

    pushBackIntercept(first)
    expect(isTopBackIntercept(first)).toBe(true)
    expect(isTopBackIntercept(second)).toBe(false)

    pushBackIntercept(second)
    expect(isTopBackIntercept(first)).toBe(false)
    expect(isTopBackIntercept(second)).toBe(true)

    removeBackIntercept(second)
    expect(isTopBackIntercept(first)).toBe(true)
    expect(isTopBackIntercept(second)).toBe(false)

    removeBackIntercept(first)
    expect(isTopBackIntercept(first)).toBe(false)
  })

  test('re-pushing an existing id moves it to the top', () => {
    const first = allocateBackInterceptId()
    const second = allocateBackInterceptId()

    pushBackIntercept(first)
    pushBackIntercept(second)
    pushBackIntercept(first)

    expect(isTopBackIntercept(first)).toBe(true)
    expect(isTopBackIntercept(second)).toBe(false)

    removeBackIntercept(first)
    removeBackIntercept(second)
  })
})
