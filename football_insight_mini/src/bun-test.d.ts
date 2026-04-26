declare module 'bun:test' {
  export const describe: (label: string, body: () => void) => void
  export const test: (label: string, body: () => void | Promise<void>) => void
  export const expect: (value: unknown) => {
    toBe: (expected: unknown) => void
    toEqual: (expected: unknown) => void
    toBeNull: () => void
  }
}
