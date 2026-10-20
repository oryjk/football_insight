import { afterEach, describe, expect, test } from 'bun:test'
import { createStudentOrder, getStudentConfig, getStudentOrders } from './student'
import { AUTH_TOKEN_STORAGE_KEY } from '../utils/authStorage'

const globalUni = globalThis as unknown as { uni: unknown }
const previous = globalUni.uni
afterEach(() => { globalUni.uni = previous })

describe('学员接口和页面路由', () => {
  test('学员二级页面注册为自定义导航页面', async () => {
    const pages = (await import('../pages.json')).default.pages
    expect(pages.find(page => page.path === 'pages/student/index')?.style.navigationStyle).toBe('custom')
  })
  test('所有请求携带当前账号；下单只传服务 code，金额由服务端决定', async () => {
    const calls: Array<{ url: string; method: string; data: unknown; header: Record<string, string> }> = []
    globalUni.uni = {
      getStorageSync: (key: string) => key === AUTH_TOKEN_STORAGE_KEY ? 'student-token' : '',
      request: (options: { url: string; method: string; data: unknown; header: Record<string, string>; success: (result: unknown) => void }) => {
        calls.push(options)
        options.success({ statusCode: 200, data: {} })
      },
    }
    await getStudentConfig()
    await createStudentOrder('tool_upgrade')
    await getStudentOrders(2, 20)
    expect(calls[0]?.url.endsWith('/student/config')).toBe(true)
    expect(calls[1]?.method).toBe('POST')
    expect(calls[1]?.data).toEqual({ code: 'tool_upgrade' })
    expect(calls[2]?.url.endsWith('/student/orders')).toBe(true)
    expect(calls[2]?.data).toEqual({ page: 2, page_size: 20 })
    expect(calls.every(call => call.header.Authorization === 'Bearer student-token')).toBe(true)
  })
})
