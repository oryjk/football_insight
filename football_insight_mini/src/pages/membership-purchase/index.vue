<template>
  <page-meta :page-style="navigationPageStyle + pageLockStyle" />
  <FiSecondaryNav :shareable="false" @back="goBackFromSecondaryPage('/pages/user/index')" />
  <view class="page-root">
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page">
      <FiPanel v-if="paymentChecked && !paymentAvailable">
        <text class="notice-title">会员购买暂未开放</text>
        <text class="notice-body">{{ paymentUnavailableMessage }}</text>
      </FiPanel>

      <FiPanel>
        <FiSectionHeading kicker="选择回流看板会员档位" />
        <MembershipTierList
          v-if="productOptions.length"
          :options="productOptions"
          :selected-tier="selectedProduct?.target_tier ?? ''"
          :summaries="tierSummaries"
          @select="selectTier"
        />
        <view v-else class="tier-empty">
          <text class="notice-title">{{ emptyProductTitle }}</text>
          <text class="notice-body">{{ emptyProductBody }}</text>
        </view>
      </FiPanel>

      <MembershipBenefitsPanel
        v-if="selectedBenefits"
        :code="formatMembershipTierLabel(selectedBenefits.code)"
        :benefit-items="selectedBenefits.items"
      />

      <FiPanel flush>
        <view class="link-row" @tap="rulesSheetVisible = true">
          <text class="link-row__label">会员等级与邀请升级规则</text>
          <text class="link-row__arrow">›</text>
        </view>
      </FiPanel>
    </view>

    <view class="purchase-bar">
      <view class="agreement-line" @tap="agreementAccepted = !agreementAccepted">
        <view class="agreement-line__box" :class="{ 'agreement-line__box--checked': agreementAccepted }">
          <text v-if="agreementAccepted">✓</text>
        </view>
        <text class="agreement-line__text">我已阅读并同意</text>
        <text class="agreement-line__link" @tap.stop="openAgreement">《会员服务声明》</text>
      </view>

      <view class="purchase-bar__main">
        <view class="purchase-bar__summary">
          <view class="purchase-bar__price-line">
            <text class="purchase-bar__currency">¥</text>
            <text class="purchase-bar__price">{{ displayPrice }}</text>
          </view>
          <text class="purchase-bar__caption">{{ purchaseCaption }}</text>
        </view>
        <button
          class="purchase-btn"
          :class="{ 'purchase-btn--loading': loading, 'purchase-btn--pending': !agreementAccepted }"
          :disabled="loading || !paymentAvailable || !selectedProduct"
          @click="handlePurchase"
        >
          {{ loading ? '处理中...' : '确认开通' }}
        </button>
      </view>
    </view>

    <MembershipRulesSheet
      v-model:visible="rulesSheetVisible"
      :summary="membershipRuleSummary"
      :guides="membershipTierGuides"
      :selected-tier="selectedProduct?.target_tier ?? ''"
    />
    <MembershipAgreementSheet
      v-model:visible="agreementSheetVisible"
      :points="agreementPoints"
      :required="agreementRequired"
      @accept="acceptAgreement"
    />
  </view>
</template>

<script setup lang="ts">
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { onLoad, onShow } from '@dcloudio/uni-app'
import { ref, computed, watch } from 'vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiPanel from '../../components/FiPanel.vue'
import FiSectionHeading from '../../components/FiSectionHeading.vue'
import MembershipAgreementSheet from './components/MembershipAgreementSheet.vue'
import MembershipBenefitsPanel from './components/MembershipBenefitsPanel.vue'
import MembershipRulesSheet from './components/MembershipRulesSheet.vue'
import MembershipTierList from './components/MembershipTierList.vue'
import { buildMembershipBenefitPreview, buildMembershipTierSummary } from '../../utils/membershipBenefits'
import { goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { getMembershipProduct, createMembershipOrder, getVirtualPaymentStatus } from '../../api/payment'
import { startPaymentAttempt } from '../../composables/paymentTelemetry'
import { prepareVirtualPayment } from '../../composables/prepareVirtualPayment'
import { waitForPaidOrder } from '../../composables/confirmPaidOrder'
import { canUseVirtualPayment, requestDigitalPayment } from '../../utils/virtualPayment'
import { useRuntimeConfig } from '../../stores/runtimeConfig'
import { ApiRequestError, extractApiErrorMessage } from '../../utils/apiError'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import { useAppSession } from '../../stores/appSession'
import { reportPageActivity } from '../../utils/userActivity'
import {
  buildMembershipTierGuides,
  type MembershipTierGuide,
} from '../../utils/membershipRules'
import {
  buildMembershipPurchaseAgreement,
  filterMembershipProductOptionsForUpgrade,
  formatMembershipDuration,
  formatMembershipPrice,
  formatMembershipTierLabel,
  formatMembershipUpgradeNote,
  getMembershipPayPriceCents,
  normalizeMembershipProductOptions,
  resolveSelectedTier,
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
const rulesSheetVisible = ref(false)
const agreementSheetVisible = ref(false)
// 声明弹层是否由「确认开通」触发（决定是否显示「开通前需要先同意」提示）。
const agreementRequired = ref(false)
// 弹层打开时锁住页面滚动（小程序需由页面 page-meta 承担）。
const pageLockStyle = computed(() => rulesSheetVisible.value || agreementSheetVisible.value ? 'overflow: hidden;' : '')
const agreementPoints = computed(() => buildMembershipPurchaseAgreement(selectedProduct.value?.membership_duration_months))
const paymentAvailable = ref(false)
// 首次检查完成前不展示「暂未开放」；之后的刷新沿用上次结果，避免卡片闪现导致页面抖动。
const paymentChecked = ref(false)
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

const tierSummaries = computed<Record<string, string>>(() => Object.fromEntries(
  productOptions.value.map(option => [
    option.target_tier,
    buildMembershipTierSummary(option.target_tier, publicConfig.value?.membership_tier_rules, option.subtitle),
  ]),
))

const selectedBenefits = computed(() => buildMembershipBenefitPreview(
  selectedProduct.value?.target_tier,
  publicConfig.value?.membership_tier_rules,
))

const displayPrice = computed(() => {
  return formatMembershipPrice(getMembershipPayPriceCents(selectedProduct.value))
})
const selectedUpgradeNote = computed(() => formatMembershipUpgradeNote(selectedProduct.value))
// 价格下方的小字说明：档位 · 有效期，升级补差价时附上补差说明。
const purchaseCaption = computed(() => {
  if (!selectedProduct.value) return '请选择会员档位'
  const base = `${formatMembershipTierLabel(selectedProduct.value.target_tier)} · ${formatMembershipDuration(selectedProduct.value.membership_duration_months)}`
  return selectedUpgradeNote.value ? `${base} · ${selectedUpgradeNote.value}` : base
})

onLoad(() => {
  void loadPurchaseData()
})

onShow(() => {
  reportPageActivity('membership_purchase')
  void loadPurchaseData()
})

async function loadPurchaseData() {
  const revision = session.revision.value
  try {
    const status = await getVirtualPaymentStatus()
    if (session.revision.value !== revision) return
    paymentAvailable.value = status.available && canUseVirtualPayment()
    paymentUnavailableMessage.value = !status.available ? '购买服务暂未开放，请稍后再试。' : '请使用支持该支付功能的最新版微信小程序。'
    paymentChecked.value = true
  } catch {
    if (session.revision.value !== revision) return
    paymentAvailable.value = false
    paymentUnavailableMessage.value = '购买服务暂不可用，请稍后重试。'
    paymentChecked.value = true
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

  selectedTier.value = resolveSelectedTier(selectedTier.value, productOptions.value)
}

function selectTier(targetTier: string): void {
  selectedTier.value = targetTier
}

// 未勾选声明就点「确认开通」：打开声明弹层并提示原因，点弹层里的「同意」只勾选，不自动发起支付。
function promptAgreement(): void {
  agreementRequired.value = true
  agreementSheetVisible.value = true
}

function openAgreement(): void {
  agreementRequired.value = false
  agreementSheetVisible.value = true
}

function acceptAgreement(): void {
  agreementAccepted.value = true
  agreementSheetVisible.value = false
}

watch(session.revision, () => {
  product.value = null
  selectedTier.value = ''
  paymentAvailable.value = false
  paymentChecked.value = false
  void loadPurchaseData()
})

async function handlePurchase() {
  if (loading.value || !paymentAvailable.value) return
  if (!selectedProduct.value) return
  if (!agreementAccepted.value) {
    startPaymentAttempt('membership', selectedProduct.value.target_tier).report('agreement_prompted')
    promptAgreement()
    return
  }
  const revision = session.revision.value
  const targetTier = selectedProduct.value.target_tier
  const isCurrent = () => session.revision.value === revision
  const attempt = startPaymentAttempt('membership', targetTier)
  attempt.report('purchase_tapped')
  let telemetryStage: 'session' | 'order' | 'payment' = 'session'
  loading.value = true
  let paymentCompleted = false
  try {
    await prepareVirtualPayment()
    if (!isCurrent()) return
    telemetryStage = 'order'
    attempt.report('order_create_requested')
    const order = await createMembershipOrder(targetTier, attempt.attemptId)
    attempt.bindOrder(order.order_no)
    telemetryStage = 'payment'
    if (!isCurrent()) return
    await requestDigitalPayment(order.params, attempt)
    paymentCompleted = true
    if (!isCurrent()) return
    const confirmed = await waitForPaidOrder(order.order_no, isCurrent, attempt)
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
    if (telemetryStage === 'session') attempt.report('session_prepare_failed', { error_msg: extractApiErrorMessage(error, '购买未完成，请重试') })
    if (telemetryStage === 'order') attempt.report('order_create_failed', { error_msg: extractApiErrorMessage(error, '购买未完成，请重试'), detail: error instanceof ApiRequestError ? { http_status: error.statusCode } : undefined })
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
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
  /* 底部留出固定操作栏的高度 */
  padding: calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-24) calc(240rpx + env(safe-area-inset-bottom));
  box-sizing: border-box;
}

.notice-title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}

.notice-body {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.tier-empty {
  margin-top: var(--fi-space-16);
}

.link-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fi-space-24) var(--fi-component-panel-padding);
}

.link-row__label {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-medium);
}

.link-row__arrow {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-none);
}

.purchase-bar {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 10;
  padding: var(--fi-space-16) var(--fi-space-24) calc(var(--fi-space-16) + env(safe-area-inset-bottom));
  background: var(--fi-color-surface);
  border-top: var(--fi-border-divider);
  box-shadow: 0 -12rpx 32rpx rgba(var(--fi-primitive-ink-rgb), 0.04);
}

.agreement-line {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.agreement-line__box {
  width: 28rpx;
  height: 28rpx;
  border-radius: var(--fi-radius-round);
  border: 2rpx solid var(--fi-color-divider);
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  color: var(--fi-color-surface);
  font-size: var(--fi-font-18);
}

.agreement-line__box--checked {
  border-color: var(--fi-color-primary);
  background: var(--fi-color-primary);
}

.agreement-line__link {
  color: var(--fi-color-text-primary);
}

.purchase-bar__main {
  margin-top: var(--fi-space-14);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-20);
}

.purchase-bar__summary {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-4);
}

.purchase-bar__price-line {
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-2);
  color: var(--fi-color-primary);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.purchase-bar__currency {
  font-size: var(--fi-font-26);
}

.purchase-bar__price {
  font-size: var(--fi-font-44);
}

.purchase-bar__caption {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.purchase-btn {
  flex-shrink: 0;
  margin: 0;
  width: 300rpx;
  height: 88rpx;
  line-height: 88rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-bold);
}

.purchase-btn[disabled] {
  background: rgba(var(--fi-primitive-red-rgb), 0.36);
  color: var(--fi-color-surface);
}

/* 未同意声明：保持灰态提示，但仍可点击以弹出说明。 */
.purchase-btn--pending {
  background: rgba(var(--fi-primitive-red-rgb), 0.36);
}

.purchase-btn--loading {
  opacity: 0.72;
}
</style>
