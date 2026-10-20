import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import { createStudentPage } from './useStudentPage'
import type { CurrentUser } from '../../types/auth'
import type { DigitalPaymentParams } from '../../types/payment'
import type { StudentConfig, StudentOrderList } from '../../types/student'

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(res => { resolve = res })
  return { promise, resolve }
}
async function flush() { for (let i = 0; i < 20; i += 1) await Promise.resolve() }
const service = { code: 'certificate_renewal' as const, title: '证书续期', description: '续期说明', price_cents: 9900, unit: '年' as const, enabled: true }
const config: StudentConfig = { contact_qr_url: 'https://example.com/contact.png', services: [service] }
const params = {} as DigitalPaymentParams
const pendingOrder = { order_no: 'student-1', code: service.code, title: service.title, amount_cents: 9900, status: 'pending' as const, paid_at: null, created_at: '2026-10-20T00:00:00Z' }
function setup() {
  const revision = ref(0)
  const restricted = ref(false)
  const hasAccessToken = ref(true)
  const currentUser = ref({ id: 'a' } as CurrentUser | null)
  const session = { revision, hasAccessToken, currentUser, status: ref<'authenticated' | 'loading' | 'error'>('authenticated'), ensureSessionReady: async () => currentUser.value }
  const review = { shouldRestrictFeatures: restricted, ensureMiniReviewReady: async () => null }
  const calls = { config: 0, orders: 0, create: 0, payment: 0, check: 0, leave: 0, email: 0 }
  const toasts: string[] = []
  const operations = {
    getConfig: async () => { calls.config++; return config },
    getOrders: async (page = 1, page_size = 20): Promise<StudentOrderList> => { calls.orders++; return { items: [pendingOrder], total: 1, page, page_size } },
    createOrder: async () => { calls.create++; return { order_no: 'student-1', params } },
    getOrderStatus: async () => { calls.check++; return { order_no: 'student-1', status: 'paid', amount_cents: 9900, paid_at: null } },
    getEmail: async () => { calls.email++; return { email: 'a@example.com' } },
    updateEmail: async (email: string) => ({ email }),
    preparePayment: async () => {},
    requestPayment: async () => { calls.payment++ },
    waitForPaidOrder: async () => true,
    supportsPayment: () => true,
    toast: (title: string) => { toasts.push(title) },
    leave: () => { calls.leave++ },
  }
  return { session, review, operations, calls, toasts, revision, restricted, currentUser }
}

describe('学员页审核、会话与付款状态', () => {
  test('账号认证进行中不读取旧账号服务，失败可重试', async () => {
    const s = setup()
    const page = createStudentPage(s.session, s.review, s.operations)
    s.session.status.value = 'loading'
    await page.enter()
    expect(s.calls.config).toBe(0)
    expect(page.accessible.value).toBe(false)
    s.session.status.value = 'authenticated'
    await page.enter()
    expect(s.calls.config).toBe(1)
  })

  test('会话读取失败会显示可重试错误，不留下未处理拒绝', async () => {
    const s = setup()
    s.session.ensureSessionReady = async () => { throw new Error('网络不可用') }
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    expect(s.calls.config).toBe(0)
    expect(page.loadError.value).toBe('网络不可用')
    expect(page.loading.value).toBe(false)
  })

  test('审核模式直达页面不请求任何服务或邮箱；豁免有效状态可正常加载', async () => {
    const s = setup()
    s.restricted.value = true
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    expect(s.calls).toMatchObject({ config: 0, orders: 0, email: 0, leave: 1 })
    expect(page.accessible.value).toBe(false)
    s.restricted.value = false
    await page.enter()
    expect(page.accessible.value).toBe(true)
    expect(s.calls.config).toBe(1)
  })

  test('未登录直达页面不读取私有服务与购买记录', async () => {
    const s = setup()
    s.session.hasAccessToken.value = false
    s.currentUser.value = null
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    expect(s.calls.config).toBe(0)
    expect(s.calls.orders).toBe(0)
  })

  test('迟到的旧账号配置与邮箱不会发布到新账号', async () => {
    const s = setup()
    const pending = deferred<StudentConfig>()
    s.operations.getConfig = () => pending.promise
    const page = createStudentPage(s.session, s.review, s.operations)
    const load = page.enter()
    await flush()
    s.revision.value++
    page.reset()
    pending.resolve(config)
    await load
    expect(page.config.value).toBeNull()
    expect(page.email.value).toBe('')
    expect(page.orders.value).toEqual([])
  })

  test('支付回调成功仍须服务器确认，超时保留待确认订单，可重新进入查单', async () => {
    const s = setup()
    s.operations.waitForPaidOrder = async () => false
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    await page.purchase(service.code)
    expect(s.toasts[0]).toBe('付款确认中，请稍后在购买记录中刷新')
    expect(page.orders.value[0]?.status).toBe('pending')
    await page.enter()
    await page.checkOrder('student-1')
    expect(s.calls.check).toBe(1)
    expect(s.toasts[1]).toBe('付款已确认，请联系负责人办理')
  })

  test('双击只创建一个订单，支付期间切账号也不能再次支付或发布旧账号成功', async () => {
    const s = setup()
    const pending = deferred<void>()
    s.operations.requestPayment = async () => { s.calls.payment++; await pending.promise }
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    const purchase = page.purchase(service.code)
    await flush()
    await page.purchase(service.code)
    expect(s.calls.create).toBe(1)
    s.revision.value++
    page.reset()
    await page.enter()
    await page.purchase(service.code)
    expect(s.calls.create).toBe(1)
    pending.resolve()
    await purchase
    expect(s.toasts).toEqual([])
    expect(page.paymentBusy.value).toBe(false)
  })

  test('支付准备期间账号切换不创建订单', async () => {
    const s = setup()
    const pending = deferred<void>()
    s.operations.preparePayment = () => pending.promise
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    const purchase = page.purchase(service.code)
    s.revision.value++
    pending.resolve()
    await purchase
    expect(s.calls.create).toBe(0)
  })

  test('支付取消单独提示并刷新订单，不显示付款成功', async () => {
    const s = setup()
    s.operations.requestPayment = async () => { throw new Error('支付已取消') }
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    await page.purchase(service.code)
    expect(s.toasts).toEqual(['支付已取消'])
    expect(s.calls.orders).toBe(2)
  })

  test('未配置二维码或禁用服务不会发起付款', async () => {
    const s = setup()
    s.operations.getConfig = async () => ({ ...config, contact_qr_url: '' })
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    await page.purchase(service.code)
    expect(s.calls.create).toBe(0)
    expect(page.contactQrUrl.value).toBe('')
  })

  test('邮箱保存重复提交合并，旧会话完成不覆盖新会话或提示成功', async () => {
    const s = setup()
    const pending = deferred<{ email: string }>()
    let saves = 0
    s.operations.updateEmail = () => { saves++; return pending.promise }
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    page.emailVisible.value = true
    const save = page.saveEmail('new@example.com')
    await page.saveEmail('other@example.com')
    expect(saves).toBe(1)
    s.revision.value++
    page.reset()
    pending.resolve({ email: 'old-account@example.com' })
    await save
    expect(page.email.value).toBe('')
    expect(s.toasts).toEqual([])
  })

  test('邮箱保存后，之前的慢速读取不能回写旧邮箱', async () => {
    const s = setup()
    const pending = deferred<{ email: string }>()
    s.operations.getEmail = () => pending.promise
    const page = createStudentPage(s.session, s.review, s.operations)
    const load = page.enter()
    await flush()
    await page.saveEmail('new@example.com')
    pending.resolve({ email: 'old@example.com' })
    await load
    expect(page.email.value).toBe('new@example.com')
  })

  test('审核限制在准备支付期间生效，禁止创建订单', async () => {
    const s = setup()
    const pending = deferred<void>()
    s.operations.preparePayment = () => pending.promise
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    const purchase = page.purchase(service.code)
    s.restricted.value = true
    pending.resolve()
    await purchase
    expect(s.calls.create).toBe(0)
    expect(s.toasts).toEqual([])
  })

  test('离开销毁页面后，迟到请求不能发布订单和配置', async () => {
    const s = setup()
    const pending = deferred<StudentConfig>()
    s.operations.getConfig = () => pending.promise
    const page = createStudentPage(s.session, s.review, s.operations)
    const load = page.enter()
    await flush()
    page.dispose()
    pending.resolve(config)
    await load
    expect(page.config.value).toBeNull()
    expect(page.orders.value).toEqual([])
  })

  test('当前账号可保存提醒邮箱', async () => {
    const s = setup()
    const page = createStudentPage(s.session, s.review, s.operations)
    await page.enter()
    page.emailVisible.value = true
    await page.saveEmail(' new@example.com ')
    expect(page.email.value).toBe('new@example.com')
    expect(page.emailVisible.value).toBe(false)
    expect(page.emailSaving.value).toBe(false)
    expect(s.toasts).toEqual(['邮箱已保存'])
  })
})
