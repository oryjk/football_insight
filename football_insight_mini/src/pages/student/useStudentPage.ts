import { computed, ref } from 'vue'
import { getNotificationEmail, updateNotificationEmail } from '../../api/auth'
import { getOrderStatus } from '../../api/payment'
import { createStudentOrder, getStudentConfig, getStudentOrders } from '../../api/student'
import { waitForPaidOrder } from '../../composables/confirmPaidOrder'
import { prepareVirtualPaymentForSession } from '../../composables/prepareVirtualPayment'
import type { AppSession } from '../../stores/appSession'
import type { MiniReviewStore } from '../../stores/miniReview'
import type { StudentConfig, StudentOrder, StudentServiceCode } from '../../types/student'
import { extractApiErrorMessage } from '../../utils/apiError'
import { isValidNotificationEmail } from '../../utils/notificationEmail'
import { canUseVirtualPayment, requestDigitalPayment } from '../../utils/virtualPayment'

const defaultOperations = {
  getConfig: getStudentConfig,
  getOrders: getStudentOrders,
  createOrder: createStudentOrder,
  getOrderStatus,
  getEmail: getNotificationEmail,
  updateEmail: updateNotificationEmail,
  preparePayment: prepareVirtualPaymentForSession,
  requestPayment: requestDigitalPayment,
  waitForPaidOrder,
  supportsPayment: canUseVirtualPayment,
  toast: (title: string): void => { uni.showToast({ title, icon: 'none' }) },
  leave: (): void => { uni.switchTab({ url: '/pages/user/index' }) },
}

/** 页面私有状态；所有异步写入同时校验账号、审核权限及页面生命周期。 */
export function createStudentPage(
  session: Pick<AppSession, 'revision' | 'hasAccessToken' | 'currentUser' | 'status' | 'ensureSessionReady'>,
  review: Pick<MiniReviewStore, 'shouldRestrictFeatures' | 'ensureMiniReviewReady'>,
  operations = defaultOperations,
) {
  const granted = ref(false)
  const loading = ref(false)
  const config = ref<StudentConfig | null>(null)
  const loadError = ref('')
  const orders = ref<StudentOrder[]>([])
  const orderTotal = ref(0)
  const orderPage = ref(0)
  const ordersLoading = ref(false)
  const ordersError = ref('')
  const email = ref('')
  const emailReady = ref(false)
  const emailVisible = ref(false)
  const emailSaving = ref(false)
  const paymentBusy = ref(false)
  const payingCode = ref<StudentServiceCode | null>(null)
  const checkingOrder = ref('')
  let epoch = 0
  let loadEpoch = 0
  let orderEpoch = 0
  let emailEpoch = 0
  let disposed = false
  const accessible = computed(() => granted.value && session.hasAccessToken.value
    && Boolean(session.currentUser.value) && session.status.value !== 'loading' && !review.shouldRestrictFeatures.value)
  const contactQrUrl = computed(() => {
    const url = config.value?.contact_qr_url?.trim() || ''
    return /^https:\/\/[^\s]+$/i.test(url) ? url : ''
  })
  const paymentSupported = operations.supportsPayment()

  function capture() {
    const revision = session.revision.value
    const currentEpoch = epoch
    return () => !disposed && revision === session.revision.value && currentEpoch === epoch
  }

  function reset() {
    epoch += 1
    loadEpoch += 1
    orderEpoch += 1
    emailEpoch += 1
    granted.value = false
    loading.value = false
    config.value = null
    loadError.value = ''
    orders.value = []
    orderTotal.value = 0
    orderPage.value = 0
    ordersLoading.value = false
    ordersError.value = ''
    checkingOrder.value = ''
    email.value = ''
    emailReady.value = false
    emailVisible.value = false
    emailSaving.value = false
    // 原生支付关闭前保留 busy，切换账号也不能再次唤起支付。
  }

  async function loadOrders(append = false) {
    if (!accessible.value || (append && ordersLoading.value)) return
    const current = capture()
    const requestEpoch = ++orderEpoch
    const page = append ? orderPage.value + 1 : 1
    ordersLoading.value = true
    ordersError.value = ''
    try {
      const result = await operations.getOrders(page, 20)
      if (!current() || !accessible.value || requestEpoch !== orderEpoch) return
      orders.value = append ? [...orders.value, ...result.items] : result.items
      orderTotal.value = result.total
      orderPage.value = result.page
    } catch (error) {
      if (current() && accessible.value && requestEpoch === orderEpoch) {
        ordersError.value = extractApiErrorMessage(error, '购买记录加载失败')
      }
    } finally {
      if (current() && requestEpoch === orderEpoch) ordersLoading.value = false
    }
  }

  async function enter() {
    const current = capture()
    const requestEpoch = ++loadEpoch
    loading.value = true
    loadError.value = ''
    // 认证意图尚未发布新账号时，不用旧 token 读取服务。状态结束后页面会重新进入。
    if (session.status.value === 'loading') return
    // 审核状态按当前账号查询，沿用后端审核豁免，不自行判断名单。
    try {
      await Promise.all([session.ensureSessionReady(), review.ensureMiniReviewReady({ force: true })])
    } catch (error) {
      if (current() && requestEpoch === loadEpoch) {
        loading.value = false
        loadError.value = extractApiErrorMessage(error, '账号信息加载失败，请重试')
      }
      return
    }
    if (!current() || requestEpoch !== loadEpoch) return
    if (!session.hasAccessToken.value || !session.currentUser.value || review.shouldRestrictFeatures.value) {
      reset()
      operations.leave()
      return
    }
    granted.value = true
    const requestEmailEpoch = emailEpoch
    const [configResult, emailResult] = await Promise.allSettled([
      operations.getConfig(), operations.getEmail(), loadOrders(),
    ])
    if (!current() || !accessible.value || requestEpoch !== loadEpoch) return
    if (configResult.status === 'fulfilled') config.value = configResult.value
    else loadError.value = extractApiErrorMessage(configResult.reason, '服务信息加载失败')
    if (emailResult.status === 'fulfilled' && !emailSaving.value && requestEmailEpoch === emailEpoch) {
      email.value = emailResult.value.email?.trim() || ''
      emailReady.value = true
    }
    loading.value = false
  }

  async function saveEmail(value: string) {
    if (!accessible.value || emailSaving.value || !isValidNotificationEmail(value)) return
    const current = capture()
    const isCurrent = () => current() && accessible.value
    emailEpoch += 1
    emailSaving.value = true
    try {
      const result = await operations.updateEmail(value.trim())
      if (!isCurrent()) return
      email.value = result.email?.trim() || value.trim()
      emailReady.value = true
      emailVisible.value = false
      operations.toast('邮箱已保存')
    } catch (error) {
      if (isCurrent()) operations.toast(extractApiErrorMessage(error, '邮箱保存失败'))
    } finally {
      if (isCurrent()) emailSaving.value = false
    }
  }

  async function purchase(code: StudentServiceCode) {
    const service = config.value?.services.find(item => item.code === code)
    if (!accessible.value || paymentBusy.value || !paymentSupported || !service?.enabled
      || !service.description.trim() || !contactQrUrl.value) return
    const current = capture()
    const isCurrent = () => current() && accessible.value
    paymentBusy.value = true
    payingCode.value = code
    let created = false
    let clientPaid = false
    try {
      await operations.preparePayment(session)
      if (!isCurrent()) return
      const order = await operations.createOrder(code)
      created = true
      if (!isCurrent()) return
      await operations.requestPayment(order.params)
      clientPaid = true
      if (!isCurrent()) return
      const confirmed = await operations.waitForPaidOrder(order.order_no, isCurrent)
      if (!isCurrent()) return
      operations.toast(confirmed ? '付款已确认，请联系负责人办理' : '付款确认中，请稍后在购买记录中刷新')
    } catch (error) {
      if (isCurrent()) operations.toast(clientPaid
        ? '付款确认中，请稍后在购买记录中刷新'
        : extractApiErrorMessage(error, '购买未完成，请重试'))
    } finally {
      if (isCurrent() && created) await loadOrders()
      paymentBusy.value = false
      payingCode.value = null
    }
  }

  async function checkOrder(orderNo: string) {
    if (!accessible.value || checkingOrder.value || paymentBusy.value
      || !orders.value.some(item => item.order_no === orderNo && item.status === 'pending')) return
    const current = capture()
    const isCurrent = () => current() && accessible.value
    checkingOrder.value = orderNo
    try {
      const result = await operations.getOrderStatus(orderNo)
      if (!isCurrent()) return
      await loadOrders()
      if (isCurrent()) operations.toast(result.status === 'paid' ? '付款已确认，请联系负责人办理'
        : result.status === 'refunded' ? '订单已退款'
        : result.status === 'failed' ? '订单未完成' : '暂未确认付款，请稍后刷新')
    } catch (error) {
      if (isCurrent()) operations.toast(extractApiErrorMessage(error, '订单状态查询失败'))
    } finally {
      if (isCurrent()) checkingOrder.value = ''
    }
  }

  function dispose() { reset(); disposed = true }

  return {
    accessible, loading, config, loadError, contactQrUrl, orders, orderTotal, orderPage,
    ordersLoading, ordersError, email, emailReady, emailVisible, emailSaving,
    paymentBusy, payingCode, checkingOrder, paymentSupported,
    enter, loadOrders, saveEmail, purchase, checkOrder, reset, dispose,
  }
}
