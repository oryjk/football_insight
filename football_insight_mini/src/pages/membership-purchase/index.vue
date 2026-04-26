<template>
  <view v-if="systemConfigUnderReview" class="page-root" />
  <view v-else class="page-root">
    <view class="page">
      <view class="purchase-hero">
        <text class="purchase-hero__eyebrow">会员充值</text>
        <text class="purchase-hero__title">选择回流看板会员档位</text>
        <text class="purchase-hero__summary">V6 起解锁最近回流速览，支付成功后立即生效。</text>
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
            <text class="product-card__tier">{{ option.target_tier }}</text>
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
          <text class="purchase-actions__tier">{{ selectedProduct?.target_tier || '--' }}</text>
          <text class="purchase-actions__price">¥{{ displayPrice }}</text>
          <text v-if="selectedUpgradeNote" class="purchase-actions__note">{{ selectedUpgradeNote }}</text>
        </view>
        <button
          class="purchase-btn"
          :class="{ 'purchase-btn--loading': loading }"
          :disabled="loading || !agreementAccepted || !selectedProduct"
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
import { onLoad, onShow } from '@dcloudio/uni-app'
import { ref, computed } from 'vue'
import { getMembershipProduct, createMembershipOrder } from '../../api/payment'
import { getCurrentUser } from '../../api/auth'
import type { CurrentUser } from '../../types/auth'
import { getPublicSystemConfig } from '../../api/system'
import type { PublicSystemConfig } from '../../types/system'
import { extractApiErrorMessage } from '../../utils/apiError'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
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

const product = ref<Awaited<ReturnType<typeof getMembershipProduct>> | null>(null)
const publicConfig = ref<PublicSystemConfig | null>(null)
const currentUser = ref<CurrentUser | null>(null)
const loading = ref(false)
const selectedTier = ref('')
const agreementAccepted = ref(false)
const agreementPoints = MEMBERSHIP_PURCHASE_AGREEMENT_POINTS
const systemConfigUnderReview = ref(false)

const allProductOptions = computed(() => normalizeMembershipProductOptions(product.value))
const productOptions = computed(() =>
  filterMembershipProductOptionsForUpgrade(allProductOptions.value, currentUser.value?.membership_tier),
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
  loadPurchaseData()
})

onShow(() => {
  reportPageActivity('membership_purchase')
})

async function loadPurchaseData() {
  systemConfigUnderReview.value = await loadSystemConfigUnderReview()
  if (systemConfigUnderReview.value) {
    product.value = null
    publicConfig.value = null
    currentUser.value = null
    selectedTier.value = ''
    return
  }

  const [productResult, publicConfigResult, userResult] = await Promise.allSettled([
    getMembershipProduct(),
    getPublicSystemConfig(),
    getCurrentUser(),
  ])

  if (productResult.status === 'fulfilled') {
    product.value = productResult.value
  } else {
    uni.showToast({ title: extractApiErrorMessage(productResult.reason, '会员信息加载失败'), icon: 'none' })
  }

  if (publicConfigResult.status === 'fulfilled') {
    publicConfig.value = publicConfigResult.value
  }

  if (userResult.status === 'fulfilled') {
    currentUser.value = userResult.value
  }

  selectedTier.value = productOptions.value[0]?.target_tier ?? ''
}

function selectTier(targetTier: string): void {
  selectedTier.value = targetTier
}

async function handlePurchase() {
  if (loading.value) return
  if (!selectedProduct.value) {
    uni.showToast({ title: '请选择会员档位', icon: 'none' })
    return
  }
  if (!agreementAccepted.value) {
    uni.showToast({ title: '请先确认会员服务声明', icon: 'none' })
    return
  }

  loading.value = true

  try {
    const order = await createMembershipOrder(selectedProduct.value.target_tier)

    uni.requestPayment({
      provider: 'wxpay',
      timeStamp: order.params.timeStamp,
      nonceStr: order.params.nonceStr,
      package: order.params.package,
      signType: order.params.signType,
      paySign: order.params.paySign,
      success: async () => {
        uni.showToast({ title: '支付成功', icon: 'success' })
        try {
          currentUser.value = await getCurrentUser()
        } catch {
          // ignore refresh error
        }
        setTimeout(() => {
          uni.navigateBack()
        }, 1200)
      },
      fail: (err: any) => {
        const msg = err?.errMsg || '支付取消'
        if (msg.includes('cancel') || msg.includes('关闭') || msg.includes('fail')) {
          uni.showToast({ title: '支付已取消', icon: 'none' })
        } else {
          uni.showToast({ title: '支付失败', icon: 'none' })
        }
      },
    })
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '下单失败'), icon: 'none' })
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.page-root {
  min-height: 100vh;
  background-color: #f6f5f2;
  box-sizing: border-box;
}

.page {
  padding: 32rpx;
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
  color: #a87420;
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
  background: linear-gradient(180deg, #ffffff, #fbfaf7);
  border: 2rpx solid rgba(226, 220, 207, 0.9);
  box-shadow: 0 10rpx 26rpx rgba(43, 37, 26, 0.04);
}

.product-card--active {
  border-color: rgba(184, 132, 43, 0.88);
  background: linear-gradient(180deg, #fff9eb, #ffffff);
  box-shadow: 0 16rpx 34rpx rgba(184, 132, 43, 0.16);
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
  color: #9a6b1c;
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
  border: 2rpx solid rgba(184, 132, 43, 0.38);
  color: #a87420;
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
  --guide-accent: #a87420;
  --guide-bg: rgba(255, 249, 234, 0.96);
  --guide-border: rgba(236, 212, 167, 0.95);
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
  box-shadow: 0 12rpx 24rpx rgba(160, 116, 35, 0.12);
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
  background: rgba(184, 132, 43, 0.12);
  color: #9a6b1c;
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
  background: #f5efe2;
  color: #9a6b1c;
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
  background: #b8842b;
  border-color: #b8842b;
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
  color: #9a6b1c;
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
