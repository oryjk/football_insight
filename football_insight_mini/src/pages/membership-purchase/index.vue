<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav :shareable="false" @back="goBackFromSecondaryPage('/pages/user/index')" />
  <view class="page-root">
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page">
      <view class="purchase-hero">
        <text class="purchase-hero__eyebrow">会员充值</text>
        <text class="purchase-hero__title">选择回流看板会员档位</text>
        <text class="purchase-hero__summary">V6 起解锁最近回流速览，支付成功后立即生效。</text>
      </view>

      <view v-if="!paymentAvailable" class="purchase-empty-card">
        <text class="purchase-empty-card__title">会员购买暂未开放</text>
        <text class="purchase-empty-card__body">{{ paymentUnavailableMessage }}</text>
      </view>

      <view v-if="productOptions.length" class="product-grid">
        <view
          v-for="option in productOptions"
          :key="option.target_tier"
          class="product-card"
          :class="{ 'product-card--active': selectedTier === option.target_tier }"
          @tap="selectTier(option.target_tier)"
        >
          <view class="product-card__top">
            <text class="product-card__tier">{{ option.target_tier === 'V3' ? 'VIP3' : option.target_tier }}</text>
            <text class="product-card__tag">{{ option.subtitle }}</text>
          </view>
          <view class="product-card__price-row">
            <text class="product-card__currency">¥</text>
            <text class="product-card__price">{{ formatMembershipPrice(getMembershipPayPriceCents(option)) }}</text>
            <text v-if="isMembershipUpgradePrice(option)" class="product-card__original-price">
              ¥{{ formatMembershipPrice(getMembershipOriginalPriceCents(option)) }}
            </text>
          </view>
          <text v-if="isMembershipUpgradePrice(option)" class="product-card__upgrade-note">
            {{ formatMembershipUpgradeNote(option) }}
          </text>
          <text class="product-card__description">{{ option.description }}</text>
        </view>
      </view>
      <view v-else class="purchase-empty-card">
        <text class="purchase-empty-card__title">{{ emptyProductTitle }}</text>
        <text class="purchase-empty-card__body">{{ emptyProductBody }}</text>
      </view>

      <view class="membership-guide-card">
        <view class="membership-guide-card__title-row">
          <view class="membership-guide-card__info-icon">i</view>
          <text class="membership-guide-card__title">升级说明</text>
        </view>
        <text class="membership-guide-card__body">{{ membershipRuleSummary }}</text>

        <view v-if="membershipTierGuides.length" class="membership-guide-list">
          <view
            v-for="guide in membershipTierGuides"
            :key="guide.code"
            class="membership-guide-item"
            :class="[guide.toneClass, { 'membership-guide-item--selected': guide.code === selectedTier }]"
          >
            <view class="membership-guide-item__main">
              <view class="membership-guide-item__title-row">
                <text class="membership-guide-item__code">{{ guide.code }}</text>
                <text class="membership-guide-item__name">{{ guide.name }}</text>
              </view>
              <text class="membership-guide-item__condition">{{ guide.condition }}</text>
            </view>
            <view class="membership-guide-item__aside">
              <text class="membership-guide-item__refresh">{{ guide.refreshLabel }}</text>
              <text v-if="guide.code === selectedTier" class="membership-guide-item__selected">已选择</text>
            </view>
          </view>
        </view>
      </view>

      <view class="agreement-card">
        <view class="agreement-card__header">
          <text class="agreement-card__title">会员服务声明</text>
          <text class="agreement-card__badge">支付前确认</text>
        </view>
        <view class="agreement-list">
          <view
            v-for="(point, index) in agreementPoints"
            :key="`agreement-${index}`"
            class="agreement-list__item"
          >
            <text class="agreement-list__index">{{ index + 1 }}</text>
            <text class="agreement-list__text">{{ point }}</text>
          </view>
        </view>
        <view class="agreement-check" @tap="agreementAccepted = !agreementAccepted">
          <view class="agreement-check__box" :class="{ 'agreement-check__box--checked': agreementAccepted }">
            <text v-if="agreementAccepted">✓</text>
          </view>
          <text class="agreement-check__text">我已阅读并同意以上会员服务声明</text>
        </view>
      </view>

      <view class="purchase-actions">
        <view class="purchase-actions__summary">
          <text class="purchase-actions__tier">{{ selectedProduct?.target_tier === 'V3' ? 'VIP3' : selectedProduct?.target_tier || '--' }}</text>
          <text class="purchase-actions__price">¥{{ displayPrice }}</text>
          <text v-if="selectedUpgradeNote" class="purchase-actions__note">{{ selectedUpgradeNote }}</text>
        </view>
        <button
          class="purchase-btn"
          :class="{ 'purchase-btn--loading': loading }"
          :disabled="loading || !paymentAvailable || !agreementAccepted || !selectedProduct"
          @click="handlePurchase"
        >
          <text v-if="loading">处理中...</text>
          <text v-else>确认开通</text>
        </button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { onLoad, onShow } from '@dcloudio/uni-app'
import { ref, computed, watch } from 'vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { getMembershipProduct, createMembershipOrder, getVirtualPaymentStatus } from '../../api/payment'
import { prepareVirtualPayment } from '../../composables/prepareVirtualPayment'
import { waitForPaidOrder } from '../../composables/confirmPaidOrder'
import { canUseVirtualPayment, requestDigitalPayment } from '../../utils/virtualPayment'
import { useRuntimeConfig } from '../../stores/runtimeConfig'
import { extractApiErrorMessage } from '../../utils/apiError'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import { useAppSession } from '../../stores/appSession'
import { reportPageActivity } from '../../utils/userActivity'
import {
  buildMembershipTierGuides,
  type MembershipTierGuide,
} from '../../utils/membershipRules'
import {
  MEMBERSHIP_PURCHASE_AGREEMENT_POINTS,
  filterMembershipProductOptionsForUpgrade,
  formatMembershipUpgradeNote,
  formatMembershipPrice,
  getMembershipOriginalPriceCents,
  getMembershipPayPriceCents,
  isMembershipUpgradePrice,
  normalizeMembershipProductOptions,
} from './helpers'

const { navigationPageStyle } = useNavigationLayout()

const product = ref<Awaited<ReturnType<typeof getMembershipProduct>> | null>(null)
// 公共配置共享自 runtimeConfig store（60 秒缓存，失败保留上次成功值）。
const runtimeConfig = useRuntimeConfig()
const publicConfig = runtimeConfig.config
// 当前用户共享自 appSession；支付成功后强制刷新会话以拿到新会员等级。
const session = useAppSession()
const currentUser = session.currentUser
const loading = ref(false)
const selectedTier = ref('')
const agreementAccepted = ref(false)
const agreementPoints = computed(() => selectedProduct.value?.target_tier === 'V3'
  ? ['本商品开通项目现有 V3 等级，等级及权益按当前会员规则执行。', ...MEMBERSHIP_PURCHASE_AGREEMENT_POINTS.slice(1)]
  : MEMBERSHIP_PURCHASE_AGREEMENT_POINTS)
const paymentAvailable = ref(false)
const paymentUnavailableMessage = ref('请稍后再试。')

const allProductOptions = computed(() => normalizeMembershipProductOptions(product.value))
const productOptions = computed(() =>
  filterMembershipProductOptionsForUpgrade(
    allProductOptions.value,
    currentUser.value?.membership_tier,
    currentUser.value?.membership_expires_at,
  ),
)
const emptyProductTitle = computed(() =>
  allProductOptions.value.length ? '当前已是最高会员档位' : '会员价格暂未加载',
)
const emptyProductBody = computed(() =>
  allProductOptions.value.length
    ? '你当前没有需要继续购买的更高档位。'
    : '请稍后重试，会员价格和可购买档位以后端配置为准。',
)
const membershipTierGuides = computed<MembershipTierGuide[]>(() =>
  buildMembershipTierGuides(publicConfig.value?.membership_tier_rules),
)
const membershipRuleSummary = computed(() => {
  const codes = membershipTierGuides.value.map((item) => item.code).join('、')

  if (!codes) {
    return '会员升级条件和对应刷新速度以后端当前配置为准。'
  }

  return `当前开放 ${codes}。会员等级会影响余票监控刷新频率和最近回流速览可查看的时间范围，具体以后台当前配置为准。`
})

const selectedProduct = computed(() =>
  productOptions.value.find((option) => option.target_tier === selectedTier.value)
  ?? productOptions.value[0]
  ?? null,
)

const displayPrice = computed(() => {
  return formatMembershipPrice(getMembershipPayPriceCents(selectedProduct.value))
})
const selectedUpgradeNote = computed(() => formatMembershipUpgradeNote(selectedProduct.value))

onLoad(() => {
  void loadPurchaseData()
})

onShow(() => {
  reportPageActivity('membership_purchase')
  void loadPurchaseData()
})

async function loadPurchaseData() {
  const revision = session.revision.value
  paymentAvailable.value = false
  try {
    const status = await getVirtualPaymentStatus()
    if (session.revision.value !== revision) return
    paymentAvailable.value = status.available && canUseVirtualPayment()
    paymentUnavailableMessage.value = !status.available ? '购买服务暂未开放，请稍后再试。' : '请使用支持该支付功能的最新版微信小程序。'
  } catch {
    if (session.revision.value !== revision) return
    paymentUnavailableMessage.value = '购买服务暂不可用，请稍后重试。'
  }

  const [productResult] = await Promise.allSettled([
    getMembershipProduct(),
    runtimeConfig.ensureRuntimeConfig(),
    session.ensureSessionReady(),
  ])

  if (session.revision.value !== revision) return
  if (productResult.status === 'fulfilled') {
    product.value = productResult.value
  } else {
    uni.showToast({ title: extractApiErrorMessage(productResult.reason, '会员信息加载失败'), icon: 'none' })
  }

  selectedTier.value = productOptions.value[0]?.target_tier ?? ''
}

function selectTier(targetTier: string): void {
  selectedTier.value = targetTier
}

watch(session.revision, () => {
  product.value = null
  selectedTier.value = ''
  paymentAvailable.value = false
  void loadPurchaseData()
})

async function handlePurchase() {
  if (loading.value || !paymentAvailable.value) return
  if (!selectedProduct.value || !agreementAccepted.value) return
  const revision = session.revision.value
  const targetTier = selectedProduct.value.target_tier
  const isCurrent = () => session.revision.value === revision
  loading.value = true
  let paymentCompleted = false
  try {
    await prepareVirtualPayment()
    if (!isCurrent()) return
    const order = await createMembershipOrder(targetTier)
    if (!isCurrent()) return
    await requestDigitalPayment(order.params)
    paymentCompleted = true
    if (!isCurrent()) return
    const confirmed = await waitForPaidOrder(order.order_no, isCurrent)
    if (!isCurrent()) return
    if (!confirmed) {
      uni.showToast({ title: '支付已完成，会员确认中，请稍后刷新', icon: 'none' })
      return
    }
    await session.ensureSessionReady({ force: true })
    if (!isCurrent()) return
    uni.switchTab({
      url: '/pages/user/index',
      success: () => {
        if (isCurrent()) uni.showToast({ title: '会员已生效', icon: 'success' })
      },
      fail: () => {
        if (isCurrent()) uni.showToast({ title: '会员已生效，请返回「我的」查看', icon: 'none' })
      },
    })
  } catch (error) {
    if (!isCurrent()) return
    uni.showToast({ title: paymentCompleted ? '支付已完成，请稍后刷新会员状态' : extractApiErrorMessage(error, '购买未完成，请重试'), icon: 'none' })
  } finally {
    // This flag belongs to the outstanding native payment, even if the account changed.
    loading.value = false
  }
}

</script>

<style scoped>
.page-root {
  position: relative;
  min-height: 100vh;
  background: #f7f8fa;
  box-sizing: border-box;
}

.page-bg-img {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  pointer-events: none;
  z-index: 0;
}

.page-bg-fade {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  background: linear-gradient(180deg, rgba(246, 247, 244, 0.24) 0%, rgba(247, 248, 250, 0.74) 32%, #f7f8fa 58%, #f7f8fa 100%);
  pointer-events: none;
  z-index: 0;
}

.page {
  position: relative;
  z-index: 1;
  padding: calc(var(--fi-brand-nav-height) + var(--fi-space-32)) var(--fi-space-32) var(--fi-space-32);
  box-sizing: border-box;
}

.purchase-hero {
  background-color: #ffffff;
  border-radius: 24rpx;
  padding: 40rpx;
  margin-bottom: 32rpx;
  box-shadow: 0 2rpx 16rpx rgba(0, 0, 0, 0.04);
}

.purchase-hero__eyebrow {
  display: block;
  font-size: 22rpx;
  font-weight: 700;
  color: #b91c1c;
  margin-bottom: 12rpx;
}

.purchase-hero__title {
  display: block;
  font-size: 40rpx;
  font-weight: 700;
  color: #15161b;
  margin-bottom: 12rpx;
}

.purchase-hero__summary {
  display: block;
  font-size: 28rpx;
  color: #6b7280;
  line-height: 1.45;
}

.product-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 20rpx;
}

.product-card {
  min-width: 0;
  border-radius: 22rpx;
  padding: 26rpx 24rpx;
  background: linear-gradient(180deg, #ffffff, #fff7f7);
  border: 2rpx solid rgba(226, 220, 207, 0.9);
  box-shadow: 0 10rpx 26rpx rgba(43, 37, 26, 0.04);
}

.product-card--active {
  border-color: rgba(220, 38, 38, 0.42);
  background: linear-gradient(180deg, #fff5f5, #ffffff);
  box-shadow: 0 16rpx 34rpx rgba(220, 38, 38, 0.12);
}

.product-card__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12rpx;
}

.product-card__tier {
  font-size: 34rpx;
  font-weight: 700;
  color: #15161b;
  line-height: 1;
}

.product-card__tag {
  max-width: 150rpx;
  color: #991b1b;
  font-size: 20rpx;
  line-height: 1.25;
  text-align: right;
}

.product-card__price-row {
  display: flex;
  align-items: baseline;
  gap: 10rpx;
  margin-top: 24rpx;
  margin-bottom: 10rpx;
}

.product-card__currency {
  font-size: 30rpx;
  font-weight: 700;
  color: #cb544d;
  margin-right: 6rpx;
}

.product-card__price {
  font-size: 58rpx;
  font-weight: 800;
  color: #cb544d;
  line-height: 1;
}

.product-card__original-price {
  color: #a5a09a;
  font-size: 22rpx;
  font-weight: 700;
  text-decoration: line-through;
}

.product-card__upgrade-note {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  margin-bottom: 14rpx;
  padding: 7rpx 12rpx;
  border-radius: 999rpx;
  background: rgba(203, 84, 77, 0.1);
  color: #b53f38;
  font-size: 19rpx;
  font-weight: 800;
  line-height: 1.1;
  box-sizing: border-box;
}

.product-card__description {
  display: block;
  font-size: 23rpx;
  color: #4b5563;
  line-height: 1.45;
}

.purchase-empty-card {
  padding: 34rpx 30rpx;
  border-radius: 24rpx;
  background: #ffffff;
  border: 2rpx solid rgba(226, 220, 207, 0.9);
}

.purchase-empty-card__title {
  display: block;
  color: #15161b;
  font-size: 30rpx;
  font-weight: 800;
}

.purchase-empty-card__body {
  display: block;
  margin-top: 12rpx;
  color: #6b7280;
  font-size: 25rpx;
  line-height: 1.5;
}

.membership-guide-card {
  margin-top: 28rpx;
  padding: 30rpx;
  border-radius: 24rpx;
  background: #ffffff;
  border: 2rpx solid rgba(226, 220, 207, 0.9);
}

.membership-guide-card__title-row {
  display: flex;
  align-items: center;
  gap: 12rpx;
  margin-bottom: 18rpx;
}

.membership-guide-card__info-icon {
  width: 32rpx;
  height: 32rpx;
  line-height: 32rpx;
  border-radius: 50%;
  border: 2rpx solid rgba(220, 38, 38, 0.28);
  color: #b91c1c;
  font-size: 22rpx;
  font-weight: 800;
  text-align: center;
}

.membership-guide-card__title {
  color: #15161b;
  font-size: 30rpx;
  font-weight: 800;
}

.membership-guide-card__body {
  display: block;
  color: #6b6559;
  font-size: 25rpx;
  line-height: 1.65;
}

.membership-guide-list {
  display: grid;
  gap: 14rpx;
  margin-top: 24rpx;
}

.membership-guide-item {
  --guide-accent: #8b98aa;
  --guide-bg: rgba(246, 248, 251, 0.96);
  --guide-border: rgba(216, 223, 234, 0.94);
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18rpx;
  padding: 22rpx 20rpx;
  border-radius: 18rpx;
  background: var(--guide-bg);
  border: 2rpx solid var(--guide-border);
}

.membership-guide-item.tier-tone--v1 {
  --guide-accent: #66799a;
  --guide-bg: rgba(242, 246, 252, 0.96);
  --guide-border: rgba(205, 216, 236, 0.95);
}

.membership-guide-item.tier-tone--v2,
.membership-guide-item.tier-tone--v6 {
  --guide-accent: #2f8b65;
  --guide-bg: rgba(240, 253, 246, 0.96);
  --guide-border: rgba(189, 228, 207, 0.95);
}

.membership-guide-item.tier-tone--v3,
.membership-guide-item.tier-tone--v9 {
  --guide-accent: #b91c1c;
  --guide-bg: rgba(255, 241, 242, 0.96);
  --guide-border: rgba(254, 205, 211, 0.95);
}

.membership-guide-item.tier-tone--v4,
.membership-guide-item.tier-tone--v8 {
  --guide-accent: #af5843;
  --guide-bg: rgba(255, 243, 239, 0.96);
  --guide-border: rgba(239, 200, 187, 0.95);
}

.membership-guide-item.tier-tone--v5 {
  --guide-accent: #7c58b2;
  --guide-bg: rgba(247, 242, 255, 0.96);
  --guide-border: rgba(217, 203, 240, 0.95);
}

.membership-guide-item.tier-tone--v7 {
  --guide-accent: #3a69b2;
  --guide-bg: rgba(240, 247, 255, 0.96);
  --guide-border: rgba(193, 214, 243, 0.95);
}

.membership-guide-item--selected {
  box-shadow: 0 12rpx 24rpx rgba(220, 38, 38, 0.12);
}

.membership-guide-item__main {
  min-width: 0;
}

.membership-guide-item__title-row {
  display: flex;
  align-items: center;
  gap: 8rpx;
}

.membership-guide-item__code {
  color: #171a20;
  font-size: 30rpx;
  font-weight: 900;
  line-height: 1.1;
}

.membership-guide-item__name {
  color: #69717c;
  font-size: 22rpx;
  font-weight: 800;
}

.membership-guide-item__condition {
  display: block;
  margin-top: 12rpx;
  color: #2f3640;
  font-size: 23rpx;
  line-height: 1.45;
}

.membership-guide-item__aside {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10rpx;
}

.membership-guide-item__refresh,
.membership-guide-item__selected {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 112rpx;
  padding: 10rpx 16rpx;
  border-radius: 999rpx;
  background: rgba(255, 255, 255, 0.86);
  border: 2rpx solid rgba(224, 214, 194, 0.86);
  color: #7c6949;
  font-size: 22rpx;
  font-weight: 800;
  line-height: 1;
}

.membership-guide-item__selected {
  min-width: 88rpx;
  background: rgba(255, 255, 255, 0.7);
  color: var(--guide-accent);
}

.agreement-card {
  margin-top: 28rpx;
  padding: 30rpx;
  border-radius: 24rpx;
  background: #ffffff;
  border: 2rpx solid rgba(226, 220, 207, 0.9);
}

.agreement-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
  margin-bottom: 20rpx;
}

.agreement-card__title {
  font-size: 30rpx;
  font-weight: 800;
  color: #15161b;
}

.agreement-card__badge {
  padding: 8rpx 16rpx;
  border-radius: 999rpx;
  background: rgba(220, 38, 38, 0.1);
  color: #991b1b;
  font-size: 22rpx;
  font-weight: 700;
}

.agreement-list {
  display: grid;
  gap: 16rpx;
}

.agreement-list__item {
  display: flex;
  gap: 14rpx;
  align-items: flex-start;
}

.agreement-list__index {
  flex: 0 0 34rpx;
  height: 34rpx;
  line-height: 34rpx;
  border-radius: 50%;
  background: #fee2e2;
  color: #991b1b;
  font-size: 20rpx;
  font-weight: 800;
  text-align: center;
}

.agreement-list__text {
  flex: 1;
  min-width: 0;
  color: #4b5563;
  font-size: 24rpx;
  line-height: 1.55;
}

.agreement-check {
  display: flex;
  align-items: center;
  gap: 14rpx;
  margin-top: 26rpx;
  padding-top: 24rpx;
  border-top: 2rpx solid #f0eee9;
}

.agreement-check__box {
  width: 34rpx;
  height: 34rpx;
  border-radius: 8rpx;
  border: 2rpx solid #c9bda5;
  color: #ffffff;
  font-size: 22rpx;
  line-height: 34rpx;
  text-align: center;
}

.agreement-check__box--checked {
  background: #dc2626;
  border-color: #dc2626;
}

.agreement-check__text {
  color: #30343b;
  font-size: 25rpx;
  font-weight: 700;
}

.purchase-actions {
  margin-top: 32rpx;
  display: flex;
  align-items: center;
  gap: 18rpx;
}

.purchase-actions__summary {
  flex: 0 0 210rpx;
  min-width: 0;
}

.purchase-actions__tier {
  display: block;
  color: #6b7280;
  font-size: 22rpx;
}

.purchase-actions__price {
  display: block;
  color: #cb544d;
  font-size: 36rpx;
  font-weight: 800;
  line-height: 1.2;
}

.purchase-actions__note {
  display: block;
  margin-top: 4rpx;
  color: #991b1b;
  font-size: 18rpx;
  font-weight: 700;
  line-height: 1.15;
}

.purchase-btn {
  flex: 1;
  height: 96rpx;
  line-height: 96rpx;
  background: linear-gradient(135deg, #cb544d 0%, #b53f38 100%);
  color: #ffffff;
  font-size: 32rpx;
  font-weight: 600;
  border-radius: 48rpx;
  border: none;
  text-align: center;
}

.purchase-btn::after {
  border: none;
}

.purchase-btn--loading {
  opacity: 0.7;
}

.purchase-btn[disabled] {
  opacity: 0.6;
}
</style>
