<template>
  <page-meta :page-style="pageStyle" />
  <FiSecondaryNav :shareable="false" @back="goBackFromSecondaryPage('/pages/user/index')" />
  <view class="student-page">
    <template v-if="accessible">
      <view class="intro">
        <text class="eyebrow">学员服务</text>
        <text class="page-title">学员功能</text>
        <text class="muted">在线付款后，请联系负责人办理证书续期或工具升级。</text>
      </view>

      <view class="panel contact-panel">
        <text class="section-title">联系负责人</text>
        <template v-if="contactQrUrl">
          <image class="contact-qr" :src="contactQrUrl" mode="aspectFit" :show-menu-by-longpress="true" @tap="previewQr" @error="qrFailed = true" />
          <text v-if="qrFailed" class="muted">二维码加载失败，请稍后重试。</text>
          <text v-else class="muted">点击预览，长按识别或保存微信二维码</text>
          <button class="secondary-action" @tap="previewQr">查看微信二维码</button>
        </template>
        <text v-else class="muted">联系方式暂未开放，请稍后再来。</text>
      </view>

      <view class="section-heading">
        <text class="section-title">服务购买</text>
        <text class="muted">负责人线下交付</text>
      </view>
      <text v-if="loading && !config" class="muted">服务信息加载中…</text>
      <view v-if="loadError" class="panel">
        <text class="muted">{{ loadError }}</text>
        <button class="secondary-action" :disabled="loading" @tap="enter">重新加载</button>
      </view>
      <view v-for="service in config?.services || []" :key="service.code" class="panel service-card">
        <view class="section-heading">
          <text class="section-title">{{ service.title }}</text>
          <view class="price"><text class="price-value">¥{{ formatStudentPrice(service.price_cents) }}</text><text class="muted">/{{ service.unit }}</text></view>
        </view>
        <text class="description">{{ service.description || '服务说明暂未发布' }}</text>
        <button class="primary-action" :disabled="paymentBusy || !service.enabled || !service.description.trim() || !contactQrUrl || !paymentSupported" @tap="purchase(service.code)">
          {{ payingCode === service.code ? '付款处理中…' : !service.enabled || !service.description.trim() || !contactQrUrl ? '暂未开放购买' : !paymentSupported ? '请在微信小程序内购买' : '立即购买' }}
        </button>
      </view>
      <text v-if="config && !config.services.length" class="muted">暂无开放的服务</text>

      <view class="panel">
        <view class="section-heading">
          <text class="section-title">购买记录</text>
          <button class="text-action" :disabled="ordersLoading || paymentBusy" @tap="loadOrders()">{{ ordersLoading ? '加载中…' : '刷新' }}</button>
        </view>
        <text class="muted">付款确认后，请联系负责人并提供订单号。</text>
        <text v-if="ordersError" class="muted">{{ ordersError }}</text>
        <text v-else-if="!orders.length && !ordersLoading" class="empty">暂无购买记录</text>
        <view v-for="order in orders" :key="order.order_no" class="order-item">
          <view class="section-heading">
            <text class="order-title">{{ order.title }}</text>
            <text class="order-status" :class="{ 'order-status--paid': order.status === 'paid' }">{{ studentOrderStatusLabel(order.status) }}</text>
          </view>
          <text class="muted">¥{{ formatStudentPrice(order.amount_cents) }} · {{ formatStudentOrderDate(order.created_at) }}</text>
          <text class="order-number" selectable>订单号 {{ order.order_no }}</text>
          <button v-if="order.status === 'pending'" class="secondary-action" :disabled="!!checkingOrder || paymentBusy" @tap="checkOrder(order.order_no)">{{ checkingOrder === order.order_no ? '查询中…' : '查询付款结果' }}</button>
        </view>
        <button v-if="orders.length < orderTotal" class="secondary-action" :disabled="ordersLoading" @tap="loadOrders(true)">{{ ordersLoading ? '加载中…' : '查看更多记录' }}</button>
      </view>

      <view class="panel email-panel">
        <view class="email-copy">
          <text class="section-title">回流提醒邮箱</text>
          <text class="muted email-value">{{ emailReady ? formatNotificationEmailLabel(email) : '邮箱信息暂未加载，可重新填写' }}</text>
        </view>
        <button class="secondary-action" @tap="emailVisible = true">{{ email ? '编辑' : '填写' }}</button>
      </view>
    </template>
    <view v-else class="panel">
      <text class="muted">{{ loadError || '正在加载…' }}</text>
      <button v-if="loadError" class="secondary-action" @tap="enter">重试</button>
    </view>
  </view>
  <StudentNotificationEmailSheet v-if="accessible" v-model:visible="emailVisible" :email="email" :saving="emailSaving" @save="saveEmail" />
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { onHide, onShow } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import StudentNotificationEmailSheet from './components/StudentNotificationEmailSheet.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { useMiniReviewStatus } from '../../stores/miniReview'
import { goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { formatNotificationEmailLabel } from '../../utils/notificationEmail'
import { formatStudentOrderDate, formatStudentPrice, studentOrderStatusLabel } from './helpers'
import { createStudentPage } from './useStudentPage'

const session = useAppSession()
const review = useMiniReviewStatus()
const student = createStudentPage(session, review)
const { accessible, loading, config, loadError, contactQrUrl, orders, orderTotal, ordersLoading,
  ordersError, email, emailReady, emailVisible, emailSaving, paymentBusy, payingCode,
  checkingOrder, paymentSupported, enter, loadOrders, saveEmail, purchase, checkOrder } = student
const { navigationPageStyle } = useNavigationLayout()
const pageStyle = computed(() => `${navigationPageStyle.value};${emailVisible.value ? 'overflow: hidden;' : ''}`)
const qrFailed = ref(false)
watch(contactQrUrl, () => { qrFailed.value = false })
let active = false
watch(session.revision, student.reset, { flush: 'sync' })
watch([session.revision, session.status], () => {
  if (active && session.status.value !== 'loading') void enter()
})
watch(review.shouldRestrictFeatures, (restricted) => {
  if (!restricted) return
  student.reset()
  if (active) goBackFromSecondaryPage('/pages/user/index')
}, { flush: 'sync' })
function previewQr() {
  if (!accessible.value || !contactQrUrl.value) return
  uni.previewImage({ urls: [contactQrUrl.value], current: contactQrUrl.value })
}
onShow(() => { active = true; void enter() })
onHide(() => { active = false })
onUnmounted(student.dispose)
</script>

<style scoped>
.student-page { min-height: 100vh; box-sizing: border-box; padding: calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-24) calc(var(--fi-space-40) + env(safe-area-inset-bottom)); background: var(--fi-color-page-soft); display: flex; flex-direction: column; gap: var(--fi-space-20); }
.intro, .panel { display: flex; flex-direction: column; gap: var(--fi-space-16); }
.panel { padding: var(--fi-space-24); background: var(--fi-color-surface); border: var(--fi-border-card); border-radius: var(--fi-radius-lg); box-shadow: var(--fi-shadow-soft); }
.eyebrow { color: var(--fi-color-primary); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.page-title { color: var(--fi-color-text-primary); font-size: var(--fi-font-44); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-heading); }
.section-title { color: var(--fi-color-text-primary); font-size: var(--fi-font-30); font-weight: var(--fi-weight-bold); }
.muted, .order-number { color: var(--fi-color-text-caption); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.contact-panel { align-items: center; text-align: center; }
.contact-qr { width: calc(var(--fi-space-64) * 4); height: calc(var(--fi-space-64) * 4); }
.section-heading { display: flex; justify-content: space-between; align-items: center; gap: var(--fi-space-16); }
.price { display: flex; align-items: baseline; white-space: nowrap; }
.price-value { color: var(--fi-color-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); }
.description { white-space: pre-wrap; word-break: break-word; color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); line-height: var(--fi-leading-relaxed); }
.primary-action, .secondary-action, .text-action { margin: 0; border-radius: var(--fi-radius-round); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); line-height: var(--fi-leading-normal); }
.primary-action { padding: var(--fi-space-20) var(--fi-space-24); background: var(--fi-color-primary); color: var(--fi-color-surface); }
.secondary-action { padding: var(--fi-space-12) var(--fi-space-20); background: var(--fi-color-page); color: var(--fi-color-text-primary); }
.text-action { padding: 0; background: transparent; color: var(--fi-color-primary); }
.primary-action::after, .secondary-action::after, .text-action::after { border: 0; }
.primary-action[disabled], .secondary-action[disabled] { opacity: var(--fi-opacity-pressed); }
.order-item { display: flex; flex-direction: column; align-items: stretch; gap: var(--fi-space-10); padding-top: var(--fi-space-20); border-top: var(--fi-border-divider); }
.order-title { color: var(--fi-color-text-primary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.order-status { color: var(--fi-color-text-caption); font-size: var(--fi-font-22); }
.order-status--paid { color: var(--fi-color-success-accent); }
.order-number, .email-value { word-break: break-all; }
.empty { padding: var(--fi-space-20) 0; color: var(--fi-color-text-muted); font-size: var(--fi-font-24); }
.email-panel { flex-direction: row; align-items: center; justify-content: space-between; }
.email-copy { min-width: 0; display: flex; flex-direction: column; gap: var(--fi-space-10); }
.email-panel .secondary-action { flex-shrink: 0; }
</style>
