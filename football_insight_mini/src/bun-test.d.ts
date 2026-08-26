declare module 'bun:test' {
  export const describe: (label: string, body: () => void) => void
  export const test: (label: string, body: () => void | Promise<void>) => void
  export const beforeEach: (hook: () => void) => void
  export const afterEach: (hook: () => void) => void

  export interface ExpectMatchers {
    toBe: (expected: unknown) => void
    toEqual: (expected: unknown) => void
    toBeNull: () => void
    toBeInstanceOf: (constructor: Function) => void
    toMatchObject: (expected: Record<string, unknown>) => void
  }

  export const expect: (value: unknown) => ExpectMatchers & {
    resolves: ExpectMatchers
  }
}
