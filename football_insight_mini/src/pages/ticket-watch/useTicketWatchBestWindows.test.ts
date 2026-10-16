import { expect, test } from 'bun:test'
import { effectScope, ref } from 'vue'
import { createAppSession } from '../../stores/appSession'
import { AUTH_TOKEN_STORAGE_KEY } from '../../utils/authStorage'
import { useTicketWatchBestWindows } from './useTicketWatchBestWindows'
import type { TicketWatchBestWindowsResponse } from '../../types/ticketWatch'

const response = (sample_size = 5): TicketWatchBestWindowsResponse => ({ status: 'ok', access: 'preview', sample_size, generated_at: null, top_windows: [{ start_hour: 12, end_hour: 13, avg_events: 10, hit_matches: 3 }], hours: [] })
function deferred() {
  let resolve!: (value: TicketWatchBestWindowsResponse) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<TicketWatchBestWindowsResponse>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}
function setup(fetch: () => Promise<TicketWatchBestWindowsResponse>) {
 const scope = effectScope()
 const revision = ref(0)
 const underReview = ref(false)
 let now = 1000
 const windows = scope.run(() => useTicketWatchBestWindows({ revision, underReview, fetch, now: () => now }))!
 return { windows, revision, underReview, stop: () => scope.stop(), advance: (ms: number) => { now += ms } }
}
test('successful data is cached for ten minutes across page shows', async () => {
 let calls = 0
 const ctx = setup(async () => { calls++; return response() })
 await ctx.windows.show()
 expect(ctx.windows.data.value?.sample_size).toBe(5)
 ctx.windows.hide(); ctx.advance(599999)
 await ctx.windows.show()
 expect(calls).toBe(1)
 ctx.windows.hide(); ctx.advance(1)
 await ctx.windows.show()
 expect(calls).toBe(2)
 ctx.stop()
})
test('insufficient and unavailable responses hide data and failures are silent', async () => {
 for (const status of ['insufficient', 'unavailable'] as const) {
  const ctx = setup(async () => ({ ...response(), status }))
  await ctx.windows.show()
  expect(ctx.windows.data.value).toBeNull()
  expect(ctx.windows.loading.value).toBe(false)
  ctx.stop()
 }
 const ctx = setup(async () => { throw new Error('offline') })
 await ctx.windows.show()
 expect(ctx.windows.data.value).toBeNull()
 expect(ctx.windows.loading.value).toBe(false)
 ctx.stop()
})
test('session revision synchronously clears private data before old response publishes', async () => {
 const first = deferred(); const second = deferred(); let calls = 0
 const ctx = setup(() => ++calls === 1 ? first.promise : second.promise)
 const old = ctx.windows.show()
 await Promise.resolve()
 ctx.revision.value++
 expect(calls).toBe(1)
 await Promise.resolve()
 expect(calls).toBe(2)
 expect(ctx.windows.data.value).toBeNull()
 first.resolve(response(99)); await old
 expect(ctx.windows.data.value).toBeNull()
 expect(ctx.windows.loading.value).toBe(true)
 second.resolve(response(3)); await Promise.resolve(); await Promise.resolve()
 expect(ctx.windows.data.value?.sample_size).toBe(3)
 ctx.stop()
})
test('hiding or disposing during a request rejects late writes and allows another show', async () => {
 const pending = deferred(); let calls = 0
 const ctx = setup(() => ++calls === 1 ? pending.promise : Promise.resolve(response(3)))
 const old = ctx.windows.show()
 await Promise.resolve()
 ctx.windows.hide()
 pending.resolve(response(99)); await old
 expect(ctx.windows.data.value).toBeNull()
 await ctx.windows.show()
 expect(ctx.windows.data.value?.sample_size).toBe(3)
 ctx.stop()
 const pendingDisposed = deferred(); const disposed = setup(() => pendingDisposed.promise)
 const request = disposed.windows.show(); await Promise.resolve(); disposed.stop()
 pendingDisposed.resolve(response()); await request
 expect(disposed.windows.data.value).toBeNull()
})
test('review change clears data, invalidates in-flight responses and does not fetch', async () => {
 let calls = 0
 const pending = deferred()
 const ctx = setup(() => { calls++; return pending.promise })
 const old = ctx.windows.show(); await Promise.resolve(); ctx.underReview.value = true
 pending.resolve(response()); await old
 expect(ctx.windows.data.value).toBeNull()
 await ctx.windows.show()
 expect(calls).toBe(1)
 ctx.stop()
})

test('a synchronous fetch failure cannot leave a completed pending owner blocking retry', async () => {
 let calls = 0
 const ctx = setup(() => { calls++; if (calls === 1) throw new Error('sync failure'); return Promise.resolve(response()) })
 await ctx.windows.show()
 await ctx.windows.show()
 expect(calls).toBe(2)
 expect(ctx.windows.data.value?.sample_size).toBe(5)
 ctx.stop()
})

for (const action of ['logout', 'invalidate'] as const) {
 test(`real session ${action} refresh reads credentials after synchronous storage clearing`, async () => {
  const globalWithUni = globalThis as unknown as { uni?: unknown }
  const previousUni = globalWithUni.uni
  const scope = effectScope()
  const storage: Record<string, unknown> = { [AUTH_TOKEN_STORAGE_KEY]: 'old-full-token' }
  const sentAuthorization: Array<string | undefined> = []
  globalWithUni.uni = {
   getStorageSync: (key: string) => storage[key],
   setStorageSync: (key: string, value: unknown) => { storage[key] = value },
   removeStorageSync: (key: string) => { delete storage[key] },
   request: (options: { url: string; header: Record<string, string>; success: (value: unknown) => void }) => {
    let data: unknown = {}
    if (options.url.endsWith('/ticket-watch/best-windows')) {
     const authorization = options.header.Authorization
     sentAuthorization.push(authorization)
     data = { ...response(), access: authorization ? 'full' : 'preview', hours: authorization ? Array.from({ length: 24 }, (_, hour) => ({ hour, avg_events: 10, share: 1 / 24, hit_matches: 3 })) : [] }
    }
    options.success({ statusCode: 200, data })
   },
  }
  try {
   // Use the real session, API and request functions: no mocked revision/token ordering.
   const session = createAppSession()
   const windows = scope.run(() => useTicketWatchBestWindows({ revision: session.revision, underReview: ref(false) }))!
   await windows.show()
   expect(windows.data.value?.access).toBe('full')
   const logout = action === 'logout' ? session.logoutSession() : undefined
   if (action === 'invalidate') session.invalidateSession(session.revision.value)
   expect(windows.data.value).toBeNull()
   expect(storage[AUTH_TOKEN_STORAGE_KEY]).toBe(undefined)
   await logout
   await windows.show()
   expect(sentAuthorization).toEqual(['Bearer old-full-token', undefined])
   expect(windows.data.value?.access).toBe('preview')
   expect(windows.data.value?.hours).toEqual([])
  } finally {
   scope.stop()
   globalWithUni.uni = previousUni
  }
 })
}

test('a hidden page never starts a deferred fetch whose owner was invalidated', async () => {
 let calls = 0
 const ctx = setup(async () => { calls++; return response() })
 const pending = ctx.windows.show()
 ctx.windows.hide()
 await pending
 expect(calls).toBe(0)
 expect(ctx.windows.data.value).toBeNull()
 ctx.stop()
})
